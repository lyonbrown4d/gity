package job

import (
	"context"
	"encoding/base64"
	"errors"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/samber/oops"
)

type UploadArtifactInput struct {
	Name          string `json:"name"`
	FileName      string `json:"file_name"`
	FilePath      string `json:"file_path"`
	ContentType   string `json:"content_type"`
	ContentBase64 string `json:"content_base64"`
}

type ProjectJobArtifactContent struct {
	Artifact      cidomain.ProjectJobArtifact `json:"artifact"`
	ContentBase64 string                      `json:"content_base64"`
}

type preparedArtifactUpload struct {
	FileName    string
	ContentType string
	Content     []byte
}

func (s *Service) ListProjectJobArtifacts(ctx context.Context, projectID, jobID int64) ([]cidomain.ProjectJobArtifact, error) {
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return nil, err
	}
	items, err := s.artifactRepo.ListByProjectJobID(ctx, projectID, jobID)
	if err != nil {
		return nil, oops.In("job").With("project_id", projectID, "job_id", jobID).Wrapf(err, "list project job artifacts")
	}
	return items.Values(), nil
}

func (s *Service) GetProjectJobArtifactContent(ctx context.Context, projectID, jobID, artifactID int64) (ProjectJobArtifactContent, error) {
	if _, err := s.GetProjectJob(ctx, projectID, jobID); err != nil {
		return ProjectJobArtifactContent{}, err
	}
	artifact, err := s.artifactRepo.GetByProjectJobAndID(ctx, projectID, jobID, artifactID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return ProjectJobArtifactContent{}, apperror.NotFound("project job artifact not found", err)
		}
		return ProjectJobArtifactContent{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifactID).Wrapf(err, "get project job artifact")
	}
	content, err := s.storage.Load(ctx, artifact.StorageKey)
	if err != nil {
		return ProjectJobArtifactContent{}, apperror.NotFound("project job artifact content not found", err)
	}
	return ProjectJobArtifactContent{Artifact: artifact, ContentBase64: base64.StdEncoding.EncodeToString(content)}, nil
}

func (s *Service) UploadProjectJobArtifact(ctx context.Context, projectID, jobID int64, input UploadArtifactInput) (cidomain.ProjectJobArtifact, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, apperror.NotFound("project not found", err)
	}
	if _, jobErr := s.GetProjectJob(ctx, projectID, jobID); jobErr != nil {
		return cidomain.ProjectJobArtifact{}, jobErr
	}
	payload, err := prepareArtifactUpload(projectID, jobID, input)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	artifact, err := s.createProjectJobArtifact(ctx, projectID, jobID, input, payload)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	storageKey, err := s.storeProjectJobArtifact(ctx, project, jobID, artifact, payload)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, err
	}
	return s.markProjectJobArtifactStored(ctx, projectID, jobID, artifact, payload, storageKey)
}

func prepareArtifactUpload(projectID, jobID int64, input UploadArtifactInput) (preparedArtifactUpload, error) {
	fileName := strings.TrimSpace(input.FileName)
	if fileName == "" {
		return preparedArtifactUpload{}, apperror.BadRequest("artifact file_name is required", oops.In("job").With("project_id", projectID, "job_id", jobID).New("artifact file_name is required"))
	}
	if strings.TrimSpace(input.ContentBase64) == "" {
		return preparedArtifactUpload{}, apperror.BadRequest("artifact content_base64 is required", oops.In("job").With("project_id", projectID, "job_id", jobID, "file_name", fileName).New("artifact content_base64 is required"))
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return preparedArtifactUpload{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "file_name", fileName).Wrapf(err, "decode artifact content")
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = storageports.DetectContentType(fileName, content)
	}
	return preparedArtifactUpload{FileName: fileName, ContentType: contentType, Content: content}, nil
}

func (s *Service) createProjectJobArtifact(ctx context.Context, projectID, jobID int64, input UploadArtifactInput, payload preparedArtifactUpload) (cidomain.ProjectJobArtifact, error) {
	artifact, err := s.artifactRepo.Create(ctx, storageports.CreateProjectJobArtifactInput{
		ProjectID:    projectID,
		ProjectJobID: jobID,
		Name:         input.Name,
		FileName:     payload.FileName,
		FilePath:     input.FilePath,
		ContentType:  payload.ContentType,
	})
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "file_name", payload.FileName).Wrapf(err, "create project job artifact")
	}
	return artifact, nil
}

func (s *Service) storeProjectJobArtifact(ctx context.Context, project projectdomain.Project, jobID int64, artifact cidomain.ProjectJobArtifact, payload preparedArtifactUpload) (string, error) {
	storageKey, err := s.storage.SavePipelineArtifact(ctx, project.FullPath, jobID, artifact.ID, payload.FileName, payload.Content, payload.ContentType)
	if err != nil {
		if cleanupErr := s.artifactRepo.DeleteByID(ctx, artifact.ID); cleanupErr != nil {
			return "", oops.In("job").With("project_id", project.ID, "job_id", jobID, "artifact_id", artifact.ID).Wrapf(oops.Join(err, cleanupErr), "save project job artifact and cleanup record")
		}
		return "", oops.In("job").With("project_id", project.ID, "job_id", jobID, "artifact_id", artifact.ID).Wrapf(err, "save project job artifact")
	}
	return storageKey, nil
}

func (s *Service) markProjectJobArtifactStored(ctx context.Context, projectID, jobID int64, artifact cidomain.ProjectJobArtifact, payload preparedArtifactUpload, storageKey string) (cidomain.ProjectJobArtifact, error) {
	if storeErr := s.artifactRepo.MarkStored(ctx, artifact.ID, storageports.StoreProjectJobArtifactInput{ContentType: payload.ContentType, ByteSize: int64(len(payload.Content)), StorageKey: storageKey}); storeErr != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifact.ID, "storage_key", storageKey).Wrapf(storeErr, "mark project job artifact stored")
	}
	stored, err := s.artifactRepo.GetByID(ctx, artifact.ID)
	if err != nil {
		return cidomain.ProjectJobArtifact{}, oops.In("job").With("project_id", projectID, "job_id", jobID, "artifact_id", artifact.ID).Wrapf(err, "load stored project job artifact")
	}
	return stored, nil
}
