package auth

import (
	"context"
	"encoding/base64"
	"fmt"
	"strings"

	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace_member"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	"github.com/arcgolabs/authx"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	setx "github.com/arcgolabs/collectionx/set"
)

var (
	projectReadVisibilityPolicies = mappingx.NewMapFrom(map[string]string{
		"public":   "project_public_read",
		"internal": "project_internal_read",
	})
	projectWriteRoles = setx.NewSet("developer", "maintainer", "owner")
)

type ProjectScope struct {
	ID          int64
	NamespaceID int64
	Visibility  string
}

func newEngine(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *namespacememberrepo.Repository) *authx.Engine {
	provider := authx.NewAuthenticationProviderFunc[TokenCredential](func(ctx context.Context, credential TokenCredential) (authx.AuthenticationResult, error) {
		record, err := tokenRepository.GetByToken(ctx, credential.Token)
		if err != nil {
			return authx.AuthenticationResult{}, err
		}
		user, err := userRepository.GetByID(ctx, record.UserID)
		if err != nil {
			return authx.AuthenticationResult{}, err
		}
		return authx.AuthenticationResult{Principal: Principal{UserID: user.ID, Username: user.Username}}, nil
	})

	authorizer := authx.AuthorizerFunc(func(ctx context.Context, input authx.AuthorizationModel) (authx.Decision, error) {
		principal, ok := input.Principal.(Principal)
		if !ok {
			return authx.Decision{Allowed: false, Reason: "invalid_principal"}, nil
		}
		namespaceID, _ := input.Context.Get("namespace_id")
		visibility, _ := input.Context.Get("visibility")
		namespaceIDValue, _ := namespaceID.(int64)
		visibilityValue, _ := visibility.(string)

		switch input.Action {
		case "project.read":
			if policyID, ok := projectReadVisibilityPolicies.Get(visibilityValue); ok {
				return authx.Decision{Allowed: true, PolicyID: policyID}, nil
			}
			if _, err := memberRepository.FindByNamespaceAndUser(ctx, namespaceIDValue, principal.UserID); err == nil {
				return authx.Decision{Allowed: true, PolicyID: "project_private_read"}, nil
			}
		case "project.write":
			member, err := memberRepository.FindByNamespaceAndUser(ctx, namespaceIDValue, principal.UserID)
			if err == nil {
				if projectWriteRoles.Contains(strings.TrimSpace(member.Role)) {
					return authx.Decision{Allowed: true, PolicyID: "project_write"}, nil
				}
			}
		}
		return authx.Decision{Allowed: false, Reason: "deny"}, nil
	})

	return authx.NewEngine(
		authx.WithAuthenticationManager(authx.NewProviderManager(provider)),
		authx.WithAuthorizer(authorizer),
	)
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
		return Principal{}, false, err
	}
	principal, ok := result.Principal.(Principal)
	if !ok {
		return Principal{}, false, fmt.Errorf("unexpected principal type")
	}
	return principal, true, nil
}

func (r *Runtime) CanReadProject(ctx context.Context, principal Principal, project ProjectScope) (bool, error) {
	decision, err := r.Engine.Can(ctx, authx.AuthorizationModel{
		Principal: principal,
		Action:    "project.read",
		Resource:  fmt.Sprintf("project:%d", project.ID),
		Context: mappingx.NewMapFrom(map[string]any{
			"namespace_id": project.NamespaceID,
			"visibility":   strings.TrimSpace(project.Visibility),
		}),
	})
	if err != nil {
		return false, err
	}
	return decision.Allowed, nil
}

func (r *Runtime) CanWriteProject(ctx context.Context, principal Principal, project ProjectScope) (bool, error) {
	decision, err := r.Engine.Can(ctx, authx.AuthorizationModel{
		Principal: principal,
		Action:    "project.write",
		Resource:  fmt.Sprintf("project:%d", project.ID),
		Context: mappingx.NewMapFrom(map[string]any{
			"namespace_id": project.NamespaceID,
			"visibility":   strings.TrimSpace(project.Visibility),
		}),
	})
	if err != nil {
		return false, err
	}
	return decision.Allowed, nil
}
