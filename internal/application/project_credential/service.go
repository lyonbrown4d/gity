package projectcredential

import (
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"fmt"
	"strings"
	"time"

	setx "github.com/arcgolabs/collectionx/set"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	identityports "github.com/lyonbrown4d/gity/internal/application/ports"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	"github.com/samber/oops"
)

var (
	projectAccessTokenScopes = setx.NewSet(
		identity.ProjectTokenScopeReadRepository,
		identity.ProjectTokenScopeWriteRepository,
		identity.ProjectTokenScopeReadPackage,
		identity.ProjectTokenScopeWritePackage,
		identity.ProjectTokenScopeReadAPI,
		identity.ProjectTokenScopeWriteAPI,
	)
	deployTokenScopes = setx.NewSet(
		identity.ProjectTokenScopeReadRepository,
		identity.ProjectTokenScopeWriteRepository,
		identity.ProjectTokenScopeReadPackage,
		identity.ProjectTokenScopeWritePackage,
	)
)

type Service struct {
	projectRepo identityports.ProjectRepository
	tokenRepo   identityports.ProjectAccessTokenRepository
	keyRepo     identityports.ProjectDeployKeyRepository
}

type CreateTokenInput struct {
	Name            string
	Username        string
	Scopes          []string
	CreatedByUserID int64
	ExpiresAt       time.Time
}

type CreateDeployKeyInput struct {
	Title           string
	PublicKey       string
	CanPush         bool
	CreatedByUserID int64
}

type Dependencies struct {
	ProjectRepo identityports.ProjectRepository
	TokenRepo   identityports.ProjectAccessTokenRepository
	KeyRepo     identityports.ProjectDeployKeyRepository
}

func NewDependencies(projectRepo identityports.ProjectRepository, tokenRepo identityports.ProjectAccessTokenRepository, keyRepo identityports.ProjectDeployKeyRepository) Dependencies {
	return Dependencies{ProjectRepo: projectRepo, TokenRepo: tokenRepo, KeyRepo: keyRepo}
}

func NewServiceWithDependencies(dependencies Dependencies) *Service {
	return &Service{projectRepo: dependencies.ProjectRepo, tokenRepo: dependencies.TokenRepo, keyRepo: dependencies.KeyRepo}
}

func NewService(projectRepo identityports.ProjectRepository, tokenRepo identityports.ProjectAccessTokenRepository, keyRepo identityports.ProjectDeployKeyRepository) *Service {
	return NewServiceWithDependencies(NewDependencies(projectRepo, tokenRepo, keyRepo))
}

func (s *Service) ListProjectAccessTokens(ctx context.Context, projectID int64) ([]identity.ProjectAccessToken, error) {
	return s.listTokens(ctx, projectID, identity.ProjectAccessTokenKindProject)
}

func (s *Service) CreateProjectAccessToken(ctx context.Context, projectID int64, input CreateTokenInput) (identity.ProjectAccessToken, error) {
	return s.createToken(ctx, projectID, identity.ProjectAccessTokenKindProject, "gpat_", projectAccessTokenScopes, input)
}

func (s *Service) RevokeProjectAccessToken(ctx context.Context, projectID, tokenID int64) error {
	return s.revokeToken(ctx, projectID, tokenID, identity.ProjectAccessTokenKindProject)
}

func (s *Service) ListDeployTokens(ctx context.Context, projectID int64) ([]identity.ProjectAccessToken, error) {
	return s.listTokens(ctx, projectID, identity.ProjectAccessTokenKindDeploy)
}

func (s *Service) CreateDeployToken(ctx context.Context, projectID int64, input CreateTokenInput) (identity.ProjectAccessToken, error) {
	return s.createToken(ctx, projectID, identity.ProjectAccessTokenKindDeploy, "gdt_", deployTokenScopes, input)
}

func (s *Service) RevokeDeployToken(ctx context.Context, projectID, tokenID int64) error {
	return s.revokeToken(ctx, projectID, tokenID, identity.ProjectAccessTokenKindDeploy)
}

func (s *Service) ListDeployKeys(ctx context.Context, projectID int64) ([]identity.ProjectDeployKey, error) {
	if err := s.ensureProject(ctx, projectID); err != nil {
		return nil, err
	}
	items, err := s.keyRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("project_credential").With("project_id", projectID).Wrapf(err, "list deploy keys")
	}
	return items.Values(), nil
}

func (s *Service) CreateDeployKey(ctx context.Context, projectID int64, input CreateDeployKeyInput) (identity.ProjectDeployKey, error) {
	if err := s.ensureProject(ctx, projectID); err != nil {
		return identity.ProjectDeployKey{}, err
	}
	title := strings.TrimSpace(input.Title)
	publicKey := normalizePublicKey(input.PublicKey)
	if title == "" || publicKey == "" {
		return identity.ProjectDeployKey{}, apperror.BadRequest("deploy key title and public key are required", oops.In("project_credential").With("project_id", projectID).New("deploy key title and public key are required"))
	}
	if !strings.HasPrefix(publicKey, "ssh-") && !strings.HasPrefix(publicKey, "ecdsa-") && !strings.HasPrefix(publicKey, "sk-") {
		return identity.ProjectDeployKey{}, apperror.BadRequest("deploy key public key is invalid", oops.In("project_credential").With("project_id", projectID).New("deploy key public key is invalid"))
	}
	if input.CreatedByUserID <= 0 {
		return identity.ProjectDeployKey{}, apperror.BadRequest("created_by_user_id is required", oops.In("project_credential").With("project_id", projectID).New("created_by_user_id is required"))
	}
	item, err := s.keyRepo.Create(ctx, identityports.CreateProjectDeployKeyInput{
		ProjectID:       projectID,
		Title:           title,
		Fingerprint:     fingerprintPublicKey(publicKey),
		PublicKey:       publicKey,
		CanPush:         input.CanPush,
		CreatedByUserID: input.CreatedByUserID,
	})
	if err != nil {
		return identity.ProjectDeployKey{}, oops.In("project_credential").With("project_id", projectID).Wrapf(err, "create deploy key")
	}
	return item, nil
}

func (s *Service) DeleteDeployKey(ctx context.Context, projectID, keyID int64) error {
	item, err := s.keyRepo.GetByID(ctx, keyID)
	if err != nil {
		if errors.Is(err, identityports.ErrNotFound) {
			return apperror.NotFound("deploy key not found", err)
		}
		return oops.In("project_credential").With("project_id", projectID, "deploy_key_id", keyID).Wrapf(err, "load deploy key")
	}
	if item.ProjectID != projectID {
		return apperror.NotFound("deploy key not found", nil)
	}
	if err := s.keyRepo.DeleteByID(ctx, keyID); err != nil {
		return oops.In("project_credential").With("project_id", projectID, "deploy_key_id", keyID).Wrapf(err, "delete deploy key")
	}
	return nil
}

func (s *Service) listTokens(ctx context.Context, projectID int64, kind string) ([]identity.ProjectAccessToken, error) {
	if err := s.ensureProject(ctx, projectID); err != nil {
		return nil, err
	}
	items, err := s.tokenRepo.ListByProjectIDAndKind(ctx, projectID, kind)
	if err != nil {
		return nil, oops.In("project_credential").With("project_id", projectID, "kind", kind).Wrapf(err, "list project tokens")
	}
	return items.Values(), nil
}

func (s *Service) createToken(ctx context.Context, projectID int64, kind, prefix string, allowedScopes *setx.Set[string], input CreateTokenInput) (identity.ProjectAccessToken, error) {
	if err := s.ensureProject(ctx, projectID); err != nil {
		return identity.ProjectAccessToken{}, err
	}
	name := strings.TrimSpace(input.Name)
	if name == "" {
		return identity.ProjectAccessToken{}, apperror.BadRequest("token name is required", oops.In("project_credential").With("project_id", projectID, "kind", kind).New("token name is required"))
	}
	if input.CreatedByUserID <= 0 {
		return identity.ProjectAccessToken{}, apperror.BadRequest("created_by_user_id is required", oops.In("project_credential").With("project_id", projectID, "kind", kind).New("created_by_user_id is required"))
	}
	scopes, err := normalizeScopes(input.Scopes, allowedScopes)
	if err != nil {
		return identity.ProjectAccessToken{}, err
	}
	username := strings.TrimSpace(input.Username)
	if username == "" {
		username = defaultTokenUsername(kind, name)
	}
	secret, err := generateToken(prefix)
	if err != nil {
		return identity.ProjectAccessToken{}, oops.In("project_credential").With("project_id", projectID, "kind", kind).Wrapf(err, "generate project token")
	}
	item, err := s.tokenRepo.Create(ctx, identityports.CreateProjectAccessTokenInput{
		ProjectID:       projectID,
		Kind:            kind,
		Name:            name,
		Username:        username,
		Token:           secret,
		Scopes:          strings.Join(scopes, ","),
		CreatedByUserID: input.CreatedByUserID,
		ExpiresAt:       input.ExpiresAt,
	})
	if err != nil {
		return identity.ProjectAccessToken{}, oops.In("project_credential").With("project_id", projectID, "kind", kind, "name", name).Wrapf(err, "create project token")
	}
	return item, nil
}

func (s *Service) revokeToken(ctx context.Context, projectID, tokenID int64, kind string) error {
	item, err := s.tokenRepo.GetByID(ctx, tokenID)
	if err != nil {
		if errors.Is(err, identityports.ErrNotFound) {
			return apperror.NotFound("project token not found", err)
		}
		return oops.In("project_credential").With("project_id", projectID, "token_id", tokenID, "kind", kind).Wrapf(err, "load project token")
	}
	if item.ProjectID != projectID || item.Kind != kind {
		return apperror.NotFound("project token not found", nil)
	}
	if err := s.tokenRepo.RevokeByID(ctx, tokenID); err != nil {
		return oops.In("project_credential").With("project_id", projectID, "token_id", tokenID, "kind", kind).Wrapf(err, "revoke project token")
	}
	return nil
}

func (s *Service) ensureProject(ctx context.Context, projectID int64) error {
	if projectID <= 0 {
		return apperror.BadRequest("project id is required", oops.In("project_credential").With("project_id", projectID).New("project id is required"))
	}
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		if errors.Is(err, identityports.ErrNotFound) {
			return apperror.NotFound("project not found", err)
		}
		return oops.In("project_credential").With("project_id", projectID).Wrapf(err, "load project")
	}
	return nil
}

func normalizeScopes(values []string, allowed *setx.Set[string]) ([]string, error) {
	scopes := make([]string, 0, len(values))
	seen := map[string]struct{}{}
	for _, value := range values {
		scope := strings.TrimSpace(strings.ToLower(value))
		if scope == "" {
			continue
		}
		if !allowed.Contains(scope) {
			return nil, apperror.BadRequest("token scope is invalid", oops.In("project_credential").With("scope", scope).New("token scope is invalid"))
		}
		if _, ok := seen[scope]; ok {
			continue
		}
		seen[scope] = struct{}{}
		scopes = append(scopes, scope)
	}
	if len(scopes) == 0 {
		return nil, apperror.BadRequest("token scope is required", oops.In("project_credential").New("token scope is required"))
	}
	return scopes, nil
}

func defaultTokenUsername(kind, name string) string {
	slug := strings.ToLower(strings.Join(strings.Fields(name), "-"))
	if slug == "" {
		slug = "token"
	}
	if kind == identity.ProjectAccessTokenKindDeploy {
		return "deploy-" + slug
	}
	return "project-" + slug
}

func generateToken(prefix string) (string, error) {
	buf := make([]byte, 24)
	if _, err := rand.Read(buf); err != nil {
		return "", fmt.Errorf("generate token: %w", err)
	}
	return prefix + hex.EncodeToString(buf), nil
}

func normalizePublicKey(value string) string {
	return strings.Join(strings.Fields(value), " ")
}

func fingerprintPublicKey(value string) string {
	sum := sha256.Sum256([]byte(normalizePublicKey(value)))
	return "SHA256:" + base64.RawStdEncoding.EncodeToString(sum[:])
}
