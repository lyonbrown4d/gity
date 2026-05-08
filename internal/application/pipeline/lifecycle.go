package pipeline

import (
	"context"
	"errors"
	"fmt"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/DaiYuANg/gity/internal/ci/plan_dsl"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/samber/oops"
)

func (s *Service) LintPipeline(ctx context.Context, projectID int64, input LintInput) (plandsl.PipelineSpec, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return plandsl.PipelineSpec{}, apperror.NotFound("project not found", err)
	}
	return s.compileConfig(ctx, input.ConfigContent)
}

func (s *Service) CreatePipeline(ctx context.Context, projectID int64, input CreatePipelineInput) (PipelineView, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return PipelineView{}, apperror.NotFound("project not found", err)
	}
	spec, err := s.compileConfig(ctx, input.ConfigContent)
	if err != nil {
		return PipelineView{}, err
	}
	pipeline, err := s.pipelineRepo.Create(ctx, gitports.CreateProjectPipelineInput{
		ProjectID:     projectID,
		Name:          spec.Name,
		Source:        input.Source,
		RefName:       input.RefName,
		CommitSHA:     input.CommitSHA,
		Status:        gitports.ProjectPipelineStatusPending,
		ConfigSource:  input.ConfigSource,
		ConfigContent: input.ConfigContent,
	})
	if err != nil {
		return PipelineView{}, oops.In("pipeline").With("project_id", projectID, "source", input.Source, "ref", input.RefName, "commit_sha", input.CommitSHA).Wrapf(err, "create pipeline")
	}
	jobs, err := collectionlist.ReduceErrList(collectionlist.NewList(spec.Stages...), make([]PipelineJobView, 0, len(spec.Stages)), func(acc []PipelineJobView, index int, stage plandsl.StageSpec) ([]PipelineJobView, error) {
		view, enqueueErr := s.enqueueStage(ctx, project, pipeline, stage, index, initialRunAfter(stage))
		if enqueueErr != nil {
			return acc, enqueueErr
		}
		return append(acc, view), nil
	})
	if err != nil {
		return PipelineView{}, fmt.Errorf("enqueue pipeline stages: %w", err)
	}
	return PipelineView{Pipeline: pipeline, Spec: spec, Jobs: jobs}, nil
}

func (s *Service) CreatePushPipeline(ctx context.Context, projectID int64, refName, commitSHA string) (PipelineView, bool, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return PipelineView{}, false, apperror.NotFound("project not found", err)
	}
	configContent, ok, err := s.loadRepositoryConfig(ctx, project, refName, commitSHA)
	if err != nil || !ok {
		return PipelineView{}, false, err
	}
	existing, err := s.pipelineRepo.GetByProjectSourceRefCommit(ctx, projectID, pipelineSourcePush, refName, commitSHA)
	if err == nil {
		view, viewErr := s.GetPipeline(ctx, projectID, existing.ID)
		return view, false, viewErr
	}
	if !errors.Is(err, gitports.ErrNotFound) {
		return PipelineView{}, false, oops.In("pipeline").With("project_id", projectID, "source", pipelineSourcePush, "ref", refName, "commit_sha", commitSHA).Wrapf(err, "load existing push pipeline")
	}
	view, err := s.CreatePipeline(ctx, projectID, CreatePipelineInput{
		Source:        pipelineSourcePush,
		RefName:       refName,
		CommitSHA:     commitSHA,
		ConfigSource:  defaultCIConfigPath,
		ConfigContent: configContent,
	})
	if err != nil {
		return PipelineView{}, false, err
	}
	return view, true, nil
}

func (s *Service) compileConfig(ctx context.Context, content string) (plandsl.PipelineSpec, error) {
	if strings.TrimSpace(content) == "" {
		return plandsl.PipelineSpec{}, apperror.BadRequest("ci config content is required", errors.New("ci config content is required"))
	}
	spec, err := plandsl.Compile(ctx, ".gity-ci.plano", content)
	if err != nil {
		return plandsl.PipelineSpec{}, apperror.BadRequest("invalid ci plano config", err)
	}
	return spec, nil
}

func (s *Service) loadRepositoryConfig(ctx context.Context, project projectdomain.Project, refName, commitSHA string) (string, bool, error) {
	if s.gitRepo == nil {
		return "", false, apperror.Internal("git repository service is not configured")
	}
	revision := strings.TrimSpace(commitSHA)
	if revision == "" {
		revision = strings.TrimSpace(refName)
	}
	blob, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", revision, project.DefaultBranch, defaultCIConfigPath)
	if err != nil {
		if errors.Is(err, gitports.ErrPathNotFound) || errors.Is(err, gitports.ErrReferenceNotFound) || errors.Is(err, gitports.ErrEmptyRepository) {
			return "", false, nil
		}
		return "", false, oops.In("pipeline").With("project_id", project.ID, "ref", refName, "commit_sha", commitSHA, "path", defaultCIConfigPath).Wrapf(err, "load repository ci config")
	}
	if blob.Encoding != "utf-8" {
		return "", false, apperror.BadRequest("ci config must be utf-8 text", fmt.Errorf("ci config encoding: %s", blob.Encoding))
	}
	return blob.Content, true, nil
}
