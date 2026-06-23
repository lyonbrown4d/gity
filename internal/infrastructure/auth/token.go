package auth

import (
	"context"
	"encoding/base64"
	"errors"
	"slices"
	"strings"
	"time"

	"github.com/arcgolabs/authx"
	setx "github.com/arcgolabs/collectionx/set"
	identityports "github.com/lyonbrown4d/gity/internal/application/ports"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	projectaccesstokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_access_token"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
	"github.com/samber/oops"
)

var projectTokenActionScopes = map[string][]string{
	ProjectActionRead:                {identity.ProjectTokenScopeReadRepository, identity.ProjectTokenScopeWriteRepository, identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionRepositoryRead:      {identity.ProjectTokenScopeReadRepository, identity.ProjectTokenScopeWriteRepository, identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionRepositoryPush:      {identity.ProjectTokenScopeWriteRepository},
	ProjectActionPackageRead:         {identity.ProjectTokenScopeReadPackage, identity.ProjectTokenScopeWritePackage},
	ProjectActionPackageWrite:        {identity.ProjectTokenScopeWritePackage},
	ProjectActionIssueCreate:         {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionIssueWrite:          {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionIssueComment:        {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionMergeRequestCreate:  {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionMergeRequestWrite:   {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionMergeRequestComment: {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionWikiRead:            {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionJobRead:             {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionRunnerRead:          {identity.ProjectTokenScopeReadAPI, identity.ProjectTokenScopeWriteAPI},
	ProjectActionWrite:               {identity.ProjectTokenScopeWriteAPI},
	ProjectActionWikiWrite:           {identity.ProjectTokenScopeWriteAPI},
	ProjectActionJobWrite:            {identity.ProjectTokenScopeWriteAPI},
}

func NewTokenProvider(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, projectTokenRepository *projectaccesstokenrepo.Repository) authx.AuthenticationProvider {
	return authx.NewAuthenticationProviderFunc[TokenCredential](func(ctx context.Context, credential TokenCredential) (authx.AuthenticationResult, error) {
		result, found, err := authenticateUserToken(ctx, credential.Token, userRepository, tokenRepository)
		if err != nil || found {
			return result, err
		}
		return authenticateProjectToken(ctx, credential.Token, projectTokenRepository)
	})
}

func authenticateUserToken(ctx context.Context, token string, userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository) (authx.AuthenticationResult, bool, error) {
	record, err := tokenRepository.GetByToken(ctx, token)
	if err != nil {
		if errors.Is(err, identityports.ErrNotFound) {
			return authx.AuthenticationResult{}, false, nil
		}
		return authx.AuthenticationResult{}, false, oops.In("auth").Wrapf(err, "load access token")
	}
	userRecord, err := userRepository.GetByID(ctx, record.UserID)
	if err != nil {
		return authx.AuthenticationResult{}, false, oops.In("auth").With("user_id", record.UserID).Wrapf(err, "load token user")
	}
	return authx.AuthenticationResult{Principal: Principal{UserID: userRecord.ID, Username: userRecord.Username, IsSuperAdmin: userRecord.IsSuperAdmin != 0}}, true, nil
}

func authenticateProjectToken(ctx context.Context, token string, projectTokenRepository *projectaccesstokenrepo.Repository) (authx.AuthenticationResult, error) {
	if projectTokenRepository == nil {
		return authx.AuthenticationResult{}, oops.In("auth").Wrapf(identityports.ErrNotFound, "load access token")
	}
	projectToken, err := projectTokenRepository.GetByToken(ctx, token)
	if err != nil {
		return authx.AuthenticationResult{}, oops.In("auth").Wrapf(err, "load project token")
	}
	if !projectToken.Active(time.Now().UTC()) {
		return authx.AuthenticationResult{}, oops.In("auth").With("project_id", projectToken.ProjectID, "token_id", projectToken.ID).New("project token is inactive")
	}
	return authx.AuthenticationResult{Principal: Principal{
		UserID:    projectToken.CreatedByUserID,
		Username:  projectToken.Username,
		TokenKind: projectToken.Kind,
		ProjectID: projectToken.ProjectID,
		Scopes:    projectToken.Scopes,
	}}, nil
}

func projectTokenScopeAllows(action, scopes string) bool {
	allowedScopes, ok := projectTokenActionScopes[action]
	if !ok {
		return false
	}
	scopeSet := tokenScopes(scopes)
	return slices.ContainsFunc(allowedScopes, scopeSet.Contains)
}

func tokenScopes(value string) *setx.Set[string] {
	items := setx.NewSet[string]()
	for part := range strings.SplitSeq(value, ",") {
		scope := strings.TrimSpace(strings.ToLower(part))
		if scope != "" {
			items.Add(scope)
		}
	}
	return items
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
