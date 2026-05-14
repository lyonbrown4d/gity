package project

import (
	"context"
	"net/http"

	"github.com/arcgolabs/httpx"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
)

func (e *Endpoint) projectScope(ctx context.Context, projectID int64) (infraauth.ProjectScope, error) {
	item, err := e.service.GetByID(ctx, projectID)
	if err != nil {
		return infraauth.ProjectScope{}, err
	}
	return projectAuthScope(item), nil
}

func projectAuthScope(item projectdomain.Project) infraauth.ProjectScope {
	return infraauth.ProjectScope{ID: item.ID, OrganizationID: item.OrganizationID, Visibility: item.Visibility}
}

func (e *Endpoint) requireProjectCreate(ctx context.Context, authorization string, organizationID int64) error {
	principal, err := e.requirePermissionPrincipal(ctx, authorization)
	if err != nil {
		return err
	}
	if principal.IsSuperAdmin {
		return nil
	}
	if e.organizationService == nil {
		return httpx.NewError(http.StatusInternalServerError, "organization service is not configured")
	}
	allowed, err := e.organizationService.CanCreateProject(ctx, organizationID, principal.UserID)
	if err != nil {
		return err
	}
	if !allowed {
		return httpx.NewError(http.StatusForbidden, "forbidden")
	}
	return nil
}

func (e *Endpoint) requireDirectBranchPush(ctx context.Context, authorization string, projectID int64, branchName string) error {
	protection, protected, err := e.service.MatchBranchProtection(ctx, projectID, branchName)
	if err != nil || !protected {
		return err
	}
	principal, err := e.requirePermissionPrincipal(ctx, authorization)
	if err != nil {
		return err
	}
	item, err := e.service.GetByID(ctx, projectID)
	if err != nil {
		return err
	}
	if protection.RequireMergeRequest {
		return httpx.NewError(http.StatusForbidden, "protected branch requires merge request")
	}
	allowed, err := e.authRuntime.CanProjectAccessLevel(ctx, principal, projectAuthScope(item), protection.PushAccessLevel)
	if err != nil {
		return err
	}
	if !allowed {
		return httpx.NewError(http.StatusForbidden, "protected branch push access denied")
	}
	return nil
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
