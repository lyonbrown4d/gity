package project

import (
	"context"
	"net/http"
	"strconv"

	"github.com/arcgolabs/httpx"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
)

type projectPermissionAction struct {
	key    string
	action string
}

var projectPermissionActions = []projectPermissionAction{
	{key: "project.read", action: infraauth.ProjectActionRead},
	{key: "project.write", action: infraauth.ProjectActionWrite},
	{key: "project.delete", action: infraauth.ProjectActionDelete},
	{key: "project.repository.read", action: infraauth.ProjectActionRepositoryRead},
	{key: "project.repository.push", action: infraauth.ProjectActionRepositoryPush},
	{key: "project.repository.admin", action: infraauth.ProjectActionRepositoryAdmin},
	{key: "project.issues.create", action: infraauth.ProjectActionIssueCreate},
	{key: "project.issues.write", action: infraauth.ProjectActionIssueWrite},
	{key: "project.issues.comment", action: infraauth.ProjectActionIssueComment},
	{key: "project.merge_requests.create", action: infraauth.ProjectActionMergeRequestCreate},
	{key: "project.merge_requests.write", action: infraauth.ProjectActionMergeRequestWrite},
	{key: "project.merge_requests.comment", action: infraauth.ProjectActionMergeRequestComment},
	{key: "project.merge_requests.merge", action: infraauth.ProjectActionMergeRequestMerge},
	{key: "project.packages.read", action: infraauth.ProjectActionPackageRead},
	{key: "project.packages.write", action: infraauth.ProjectActionPackageWrite},
	{key: "project.wiki.read", action: infraauth.ProjectActionWikiRead},
	{key: "project.wiki.write", action: infraauth.ProjectActionWikiWrite},
	{key: "project.jobs.read", action: infraauth.ProjectActionJobRead},
	{key: "project.jobs.write", action: infraauth.ProjectActionJobWrite},
	{key: "project.runners.read", action: infraauth.ProjectActionRunnerRead},
	{key: "project.runners.admin", action: infraauth.ProjectActionRunnerAdmin},
}

func (e *Endpoint) getPermissions(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
	principal, err := e.requirePermissionPrincipal(ctx, in.Authorization)
	if err != nil {
		return nil, err
	}
	scope, err := e.projectScope(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	actions, err := e.evaluateProjectActions(ctx, principal, scope)
	if err != nil {
		return nil, err
	}
	if !actions[infraauth.ProjectActionRead] {
		return nil, httpx.NewError(http.StatusForbidden, "forbidden")
	}
	return &projectOutput{Body: projectPermissionsView{
		ProjectID:    strconv.FormatInt(scope.ID, 10),
		UserID:       strconv.FormatInt(principal.UserID, 10),
		Actions:      actions,
		Capabilities: buildProjectCapabilities(actions),
	}}, nil
}

func (e *Endpoint) requirePermissionPrincipal(ctx context.Context, authorization string) (infraauth.Principal, error) {
	if e.authRuntime == nil {
		return infraauth.Principal{}, httpx.NewError(http.StatusInternalServerError, "auth runtime is not configured")
	}
	principal, ok, err := e.authRuntime.AuthenticateHeader(ctx, authorization)
	if err != nil {
		return infraauth.Principal{}, httpx.NewError(http.StatusUnauthorized, "invalid credentials", err)
	}
	if !ok {
		return infraauth.Principal{}, httpx.NewError(http.StatusUnauthorized, "authentication required")
	}
	return principal, nil
}

func (e *Endpoint) evaluateProjectActions(ctx context.Context, principal infraauth.Principal, scope infraauth.ProjectScope) (map[string]bool, error) {
	actions := make(map[string]bool, len(projectPermissionActions))
	for _, item := range projectPermissionActions {
		allowed, err := e.authRuntime.CanProjectAction(ctx, principal, scope, item.action)
		if err != nil {
			return nil, httpx.NewError(http.StatusForbidden, "authorization failed", err)
		}
		actions[item.key] = allowed
	}
	return actions, nil
}

func buildProjectCapabilities(actions map[string]bool) map[string]bool {
	return map[string]bool{
		"can_read":              actions[infraauth.ProjectActionRead],
		"can_write":             actions[infraauth.ProjectActionWrite],
		"can_delete":            actions[infraauth.ProjectActionDelete],
		"repository_push":       actions[infraauth.ProjectActionRepositoryPush],
		"repository_admin":      actions[infraauth.ProjectActionRepositoryAdmin],
		"issue_create":          actions[infraauth.ProjectActionIssueCreate],
		"issue_write":           actions[infraauth.ProjectActionIssueWrite],
		"issue_comment":         actions[infraauth.ProjectActionIssueComment],
		"merge_request_create":  actions[infraauth.ProjectActionMergeRequestCreate],
		"merge_request_write":   actions[infraauth.ProjectActionMergeRequestWrite],
		"merge_request_comment": actions[infraauth.ProjectActionMergeRequestComment],
		"merge_request_merge":   actions[infraauth.ProjectActionMergeRequestMerge],
		"package_write":         actions[infraauth.ProjectActionPackageWrite],
		"wiki_write":            actions[infraauth.ProjectActionWikiWrite],
		"ci_read":               actions[infraauth.ProjectActionJobRead],
		"ci_write":              actions[infraauth.ProjectActionWrite],
		"job_read":              actions[infraauth.ProjectActionJobRead],
		"job_write":             actions[infraauth.ProjectActionJobWrite],
		"runner_read":           actions[infraauth.ProjectActionRunnerRead],
		"runner_admin":          actions[infraauth.ProjectActionRunnerAdmin],
		"audit_read":            actions[infraauth.ProjectActionWrite],
	}
}
