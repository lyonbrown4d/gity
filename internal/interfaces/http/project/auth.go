package project

import (
	"context"

	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
)

func (e *Endpoint) projectScope(ctx context.Context, projectID int64) (infraauth.ProjectScope, error) {
	item, err := e.service.GetByID(ctx, projectID)
	if err != nil {
		return infraauth.ProjectScope{}, err
	}
	return infraauth.ProjectScope{ID: item.ID, NamespaceID: item.NamespaceID, Visibility: item.Visibility}, nil
}

func (e *Endpoint) attachPipelineTrigger(ctx context.Context, body map[string]any, projectID int64, branchName string) {
	if e.pipelineService == nil {
		return
	}
	branch, err := e.resolvePipelineBranch(ctx, projectID, branchName)
	if err != nil || branch.LastCommitSHA == "" {
		return
	}
	view, created, triggerErr := e.pipelineService.CreatePushPipeline(ctx, projectID, branch.Name, branch.LastCommitSHA)
	if triggerErr != nil {
		body["pipeline_error"] = triggerErr.Error()
		return
	}
	if view.Pipeline.ID == 0 {
		return
	}
	body["pipeline_id"] = view.Pipeline.ID
	body["pipeline_created"] = created
}

func (e *Endpoint) resolvePipelineBranch(ctx context.Context, projectID int64, branchName string) (projectservice.Branch, error) {
	if branchName != "" {
		return e.service.GetBranch(ctx, projectID, branchName)
	}
	branches, err := e.service.ListBranches(ctx, projectID)
	if err != nil {
		return projectservice.Branch{}, err
	}
	for _, item := range branches {
		if item.IsDefault {
			return item, nil
		}
	}
	return projectservice.Branch{}, nil
}
