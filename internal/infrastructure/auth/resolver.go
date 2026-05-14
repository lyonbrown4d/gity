package auth

import (
	"context"
	"encoding/base64"
	"fmt"
	"strings"

	"github.com/arcgolabs/authx"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	setx "github.com/arcgolabs/collectionx/set"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
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

type ProjectScope struct {
	ID             int64
	OrganizationID int64
	Visibility     string
}

func newEngine(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *organizationmemberrepo.Repository) *authx.Engine {
	return authx.NewEngine(
		authx.WithAuthenticationManager(authx.NewProviderManager(newTokenProvider(userRepository, tokenRepository))),
		authx.WithAuthorizer(newProjectAuthorizer(memberRepository)),
	)
}

func newTokenProvider(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository) authx.AuthenticationProvider {
	return authx.NewAuthenticationProviderFunc[TokenCredential](func(ctx context.Context, credential TokenCredential) (authx.AuthenticationResult, error) {
		record, err := tokenRepository.GetByToken(ctx, credential.Token)
		if err != nil {
			return authx.AuthenticationResult{}, oops.In("auth").Wrapf(err, "load access token")
		}
		user, err := userRepository.GetByID(ctx, record.UserID)
		if err != nil {
			return authx.AuthenticationResult{}, oops.In("auth").With("user_id", record.UserID).Wrapf(err, "load token user")
		}
		return authx.AuthenticationResult{Principal: Principal{UserID: user.ID, Username: user.Username, IsSuperAdmin: user.IsSuperAdmin != 0}}, nil
	})
}

func newProjectAuthorizer(memberRepository *organizationmemberrepo.Repository) authx.Authorizer {
	return authx.AuthorizerFunc(func(ctx context.Context, input authx.AuthorizationModel) (authx.Decision, error) {
		principal, organizationID, visibility, ok := authorizationProjectScope(input)
		if !ok {
			return authx.Decision{Allowed: false, Reason: "invalid_project_scope"}, nil
		}
		return authorizeProject(ctx, memberRepository, input.Action, principal, organizationID, visibility), nil
	})
}

func authorizationProjectScope(input authx.AuthorizationModel) (Principal, int64, string, bool) {
	principal, ok := input.Principal.(Principal)
	if !ok {
		return Principal{}, 0, "", false
	}
	organizationID, organizationIDFound := input.Context.Get("organization_id")
	visibility, visibilityFound := input.Context.Get("visibility")
	organizationIDValue, organizationIDOK := organizationID.(int64)
	visibilityValue, visibilityOK := visibility.(string)
	return principal, organizationIDValue, visibilityValue, organizationIDFound && organizationIDOK && visibilityFound && visibilityOK
}

func authorizeProject(ctx context.Context, memberRepository *organizationmemberrepo.Repository, action string, principal Principal, organizationID int64, visibility string) authx.Decision {
	if principal.IsSuperAdmin {
		return authx.Decision{Allowed: true, PolicyID: "super_admin"}
	}
	switch action {
	case ProjectActionRead, ProjectActionRepositoryRead, ProjectActionPackageRead, ProjectActionWikiRead:
		return authorizeProjectRead(ctx, memberRepository, principal, organizationID, visibility)
	case ProjectActionJobRead, ProjectActionRunnerRead:
		return authorizeProjectRole(ctx, memberRepository, principal, organizationID, action, projectReportRoles)
	case ProjectActionWrite,
		ProjectActionRepositoryPush,
		ProjectActionIssueWrite,
		ProjectActionMergeRequestCreate,
		ProjectActionMergeRequestWrite,
		ProjectActionPackageWrite,
		ProjectActionWikiWrite,
		ProjectActionJobWrite:
		return authorizeProjectRole(ctx, memberRepository, principal, organizationID, action, projectWriteRoles)
	case ProjectActionIssueCreate, ProjectActionIssueComment, ProjectActionMergeRequestComment:
		return authorizeProjectRole(ctx, memberRepository, principal, organizationID, action, projectReportRoles)
	case ProjectActionMergeRequestMerge, ProjectActionRepositoryAdmin, ProjectActionRunnerAdmin:
		return authorizeProjectRole(ctx, memberRepository, principal, organizationID, action, projectMergeRoles)
	case ProjectActionDelete:
		return authorizeProjectRole(ctx, memberRepository, principal, organizationID, action, projectOwnerRoles)
	default:
		return authx.Decision{Allowed: false, Reason: "deny"}
	}
}

func authorizeProjectRead(ctx context.Context, memberRepository *organizationmemberrepo.Repository, principal Principal, organizationID int64, visibility string) authx.Decision {
	if policyID, ok := projectReadVisibilityPolicies.Get(visibility); ok {
		return authx.Decision{Allowed: true, PolicyID: policyID}
	}
	if _, err := memberRepository.FindByOrganizationAndUser(ctx, organizationID, principal.UserID); err == nil {
		return authx.Decision{Allowed: true, PolicyID: "project_private_read"}
	}
	return authx.Decision{Allowed: false, Reason: "deny"}
}

func authorizeProjectRole(ctx context.Context, memberRepository *organizationmemberrepo.Repository, principal Principal, organizationID int64, action string, allowedRoles *setx.Set[string]) authx.Decision {
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

func tokenFromAuthorizationHeader(value string) (string, bool) {
	value = strings.TrimSpace(value)
	if value == "" {
		return "", false
	}
	if strings.HasPrefix(strings.ToLower(value), "bearer ") {
		token := strings.TrimSpace(value[len("Bearer "):])
		return token, token != ""
	}
	if strings.HasPrefix(strings.ToLower(value), "basic ") {
		encoded := strings.TrimSpace(value[len("Basic "):])
		raw, err := base64.StdEncoding.DecodeString(encoded)
		if err != nil {
			return "", false
		}
		parts := strings.SplitN(string(raw), ":", 2)
		if len(parts) != 2 {
			return "", false
		}
		token := strings.TrimSpace(parts[1])
		return token, token != ""
	}
	return "", false
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
