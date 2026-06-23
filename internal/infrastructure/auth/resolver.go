package auth

import (
	"context"
	"fmt"
	"strings"

	"github.com/arcgolabs/authx"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	setx "github.com/arcgolabs/collectionx/set"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
	projectaccesstokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_access_token"
	projectmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_member"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
	"github.com/samber/oops"
)

var (
	projectReadVisibilityPolicies = mappingx.NewMapFrom(map[string]string{
		"public":   "project_public_read",
		"internal": "project_internal_read",
	})
	projectWriteRoles  = setx.NewSet("developer", "maintainer", "owner")
	projectMergeRoles  = setx.NewSet("maintainer", "owner")
	projectOwnerRoles  = setx.NewSet("owner")
	projectReportRoles = setx.NewSet("guest", "reporter", "developer", "maintainer", "owner")
)

const (
	ProjectActionRead                = "project.read"
	ProjectActionWrite               = "project.write"
	ProjectActionDelete              = "project.delete"
	ProjectActionRepositoryRead      = "project.repository.read"
	ProjectActionRepositoryPush      = "project.repository.push"
	ProjectActionRepositoryAdmin     = "project.repository.admin"
	ProjectActionIssueCreate         = "project.issues.create"
	ProjectActionIssueWrite          = "project.issues.write"
	ProjectActionIssueComment        = "project.issues.comment"
	ProjectActionMergeRequestCreate  = "project.merge_requests.create"
	ProjectActionMergeRequestWrite   = "project.merge_requests.write"
	ProjectActionMergeRequestComment = "project.merge_requests.comment"
	ProjectActionMergeRequestMerge   = "project.merge_requests.merge"
	ProjectActionPackageRead         = "project.packages.read"
	ProjectActionPackageWrite        = "project.packages.write"
	ProjectActionWikiRead            = "project.wiki.read"
	ProjectActionWikiWrite           = "project.wiki.write"
	ProjectActionJobRead             = "project.jobs.read"
	ProjectActionJobWrite            = "project.jobs.write"
	ProjectActionRunnerRead          = "project.runners.read"
	ProjectActionRunnerAdmin         = "project.runners.admin"
)

const (
	ProjectAccessNoOne      = "no_one"
	ProjectAccessDeveloper  = "developer"
	ProjectAccessMaintainer = "maintainer"
	ProjectAccessOwner      = "owner"
)

type ProjectScope struct {
	ID             int64
	OrganizationID int64
	Visibility     string
}

func NewEngine(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, projectTokenRepository *projectaccesstokenrepo.Repository, memberRepository *organizationmemberrepo.Repository, projectMemberRepository *projectmemberrepo.Repository) *authx.Engine {
	return authx.NewEngine(
		authx.WithAuthenticationManager(authx.NewProviderManager(NewTokenProvider(userRepository, tokenRepository, projectTokenRepository))),
		authx.WithAuthorizer(NewProjectAuthorizer(memberRepository, projectMemberRepository)),
	)
}

func NewProjectAuthorizer(memberRepository *organizationmemberrepo.Repository, projectMemberRepository *projectmemberrepo.Repository) authx.Authorizer {
	return authx.AuthorizerFunc(func(ctx context.Context, input authx.AuthorizationModel) (authx.Decision, error) {
		principal, projectID, organizationID, visibility, ok := authorizationProjectScope(input)
		if !ok {
			return authx.Decision{Allowed: false, Reason: "invalid_project_scope"}, nil
		}
		return authorizeProject(ctx, memberRepository, projectMemberRepository, input.Action, principal, projectID, organizationID, visibility), nil
	})
}

func authorizationProjectScope(input authx.AuthorizationModel) (Principal, int64, int64, string, bool) {
	principal, ok := input.Principal.(Principal)
	if !ok {
		return Principal{}, 0, 0, "", false
	}
	projectID, projectIDFound := input.Context.Get("project_id")
	organizationID, organizationIDFound := input.Context.Get("organization_id")
	visibility, visibilityFound := input.Context.Get("visibility")
	projectIDValue, projectIDOK := projectID.(int64)
	organizationIDValue, organizationIDOK := organizationID.(int64)
	visibilityValue, visibilityOK := visibility.(string)
	return principal, projectIDValue, organizationIDValue, visibilityValue, projectIDFound && projectIDOK && organizationIDFound && organizationIDOK && visibilityFound && visibilityOK
}

func authorizeProject(ctx context.Context, memberRepository *organizationmemberrepo.Repository, projectMemberRepository *projectmemberrepo.Repository, action string, principal Principal, projectID, organizationID int64, visibility string) authx.Decision {
	if principal.IsSuperAdmin {
		return authx.Decision{Allowed: true, PolicyID: "super_admin"}
	}
	if principal.ProjectID > 0 {
		return authorizeProjectToken(action, principal, projectID)
	}
	switch action {
	case ProjectActionRead, ProjectActionRepositoryRead, ProjectActionPackageRead, ProjectActionWikiRead:
		return authorizeProjectRead(ctx, memberRepository, projectMemberRepository, principal, projectID, organizationID, visibility)
	case ProjectActionJobRead, ProjectActionRunnerRead:
		return authorizeProjectRole(ctx, memberRepository, projectMemberRepository, principal, projectID, organizationID, action, projectReportRoles)
	case ProjectActionWrite,
		ProjectActionRepositoryPush,
		ProjectActionIssueWrite,
		ProjectActionMergeRequestCreate,
		ProjectActionMergeRequestWrite,
		ProjectActionPackageWrite,
		ProjectActionWikiWrite,
		ProjectActionJobWrite:
		return authorizeProjectRole(ctx, memberRepository, projectMemberRepository, principal, projectID, organizationID, action, projectWriteRoles)
	case ProjectActionIssueCreate, ProjectActionIssueComment, ProjectActionMergeRequestComment:
		return authorizeProjectRole(ctx, memberRepository, projectMemberRepository, principal, projectID, organizationID, action, projectReportRoles)
	case ProjectActionMergeRequestMerge, ProjectActionRepositoryAdmin, ProjectActionRunnerAdmin:
		return authorizeProjectRole(ctx, memberRepository, projectMemberRepository, principal, projectID, organizationID, action, projectMergeRoles)
	case ProjectActionDelete:
		return authorizeProjectRole(ctx, memberRepository, projectMemberRepository, principal, projectID, organizationID, action, projectOwnerRoles)
	default:
		return authx.Decision{Allowed: false, Reason: "deny"}
	}
}

func authorizeProjectRead(ctx context.Context, memberRepository *organizationmemberrepo.Repository, projectMemberRepository *projectmemberrepo.Repository, principal Principal, projectID, organizationID int64, visibility string) authx.Decision {
	if policyID, ok := projectReadVisibilityPolicies.Get(visibility); ok {
		return authx.Decision{Allowed: true, PolicyID: policyID}
	}
	if projectMemberRepository != nil {
		if _, err := projectMemberRepository.FindByProjectAndUser(ctx, projectID, principal.UserID); err == nil {
			return authx.Decision{Allowed: true, PolicyID: "project_member_read"}
		}
	}
	if _, err := memberRepository.FindByOrganizationAndUser(ctx, organizationID, principal.UserID); err == nil {
		return authx.Decision{Allowed: true, PolicyID: "project_private_read"}
	}
	return authx.Decision{Allowed: false, Reason: "deny"}
}

func authorizeProjectToken(action string, principal Principal, projectID int64) authx.Decision {
	if principal.ProjectID != projectID {
		return authx.Decision{Allowed: false, Reason: "project_token_project_mismatch"}
	}
	if projectTokenScopeAllows(action, principal.Scopes) {
		return authx.Decision{Allowed: true, PolicyID: "project_token_" + action}
	}
	return authx.Decision{Allowed: false, Reason: "project_token_scope_denied"}
}

func authorizeProjectRole(ctx context.Context, memberRepository *organizationmemberrepo.Repository, projectMemberRepository *projectmemberrepo.Repository, principal Principal, projectID, organizationID int64, action string, allowedRoles *setx.Set[string]) authx.Decision {
	if projectMemberRepository != nil {
		if member, err := projectMemberRepository.FindByProjectAndUser(ctx, projectID, principal.UserID); err == nil {
			if allowedRoles.Contains(strings.TrimSpace(member.Role)) {
				return authx.Decision{Allowed: true, PolicyID: "project_member_" + action}
			}
		}
	}
	member, err := memberRepository.FindByOrganizationAndUser(ctx, organizationID, principal.UserID)
	if err != nil {
		return authx.Decision{Allowed: false, Reason: "deny"}
	}
	role := strings.TrimSpace(member.Role)
	if allowedRoles.Contains(role) {
		return authx.Decision{Allowed: true, PolicyID: action}
	}
	return authx.Decision{Allowed: false, Reason: "deny"}
}

func TokenFromAuthorizationHeader(value string) (string, bool) {
	return tokenFromAuthorizationHeader(value)
}

func (r *Runtime) AuthenticateHeader(ctx context.Context, authorization string) (Principal, bool, error) {
	token, ok := tokenFromAuthorizationHeader(authorization)
	if !ok {
		return Principal{}, false, nil
	}
	result, err := r.Engine.Check(ctx, TokenCredential{Token: token})
	if err != nil {
		return Principal{}, false, oops.In("auth").Wrapf(err, "authenticate token")
	}
	principal, ok := result.Principal.(Principal)
	if !ok {
		return Principal{}, false, oops.In("auth").With("principal_type", fmt.Sprintf("%T", result.Principal)).New("unexpected principal type")
	}
	return principal, true, nil
}

func (r *Runtime) CanReadProject(ctx context.Context, principal Principal, project ProjectScope) (bool, error) {
	return r.CanProjectAction(ctx, principal, project, ProjectActionRead)
}

func (r *Runtime) CanWriteProject(ctx context.Context, principal Principal, project ProjectScope) (bool, error) {
	return r.CanProjectAction(ctx, principal, project, ProjectActionWrite)
}

func (r *Runtime) CanProjectAction(ctx context.Context, principal Principal, project ProjectScope, action string) (bool, error) {
	decision, err := r.Engine.Can(ctx, authx.AuthorizationModel{
		Principal: principal,
		Action:    action,
		Resource:  fmt.Sprintf("project:%d", project.ID),
		Context: mappingx.NewMapFrom(map[string]any{
			"project_id":      project.ID,
			"organization_id": project.OrganizationID,
			"visibility":      strings.TrimSpace(project.Visibility),
		}),
	})
	if err != nil {
		return false, oops.In("auth").
			With("project_id", project.ID, "organization_id", project.OrganizationID, "user_id", principal.UserID, "action", action).
			Wrapf(err, "authorize project action")
	}
	return decision.Allowed, nil
}

func (r *Runtime) CanProjectAccessLevel(ctx context.Context, principal Principal, project ProjectScope, accessLevel string) (bool, error) {
	switch strings.TrimSpace(strings.ToLower(accessLevel)) {
	case ProjectAccessNoOne:
		return false, nil
	case ProjectAccessDeveloper:
		return r.CanProjectAction(ctx, principal, project, ProjectActionRepositoryPush)
	case ProjectAccessMaintainer:
		return r.CanProjectAction(ctx, principal, project, ProjectActionRepositoryAdmin)
	case ProjectAccessOwner:
		return r.CanProjectAction(ctx, principal, project, ProjectActionDelete)
	default:
		return false, nil
	}
}
