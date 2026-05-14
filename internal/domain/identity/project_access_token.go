package identity

import "time"

const (
	ProjectAccessTokenKindProject = "project_access"
	ProjectAccessTokenKindDeploy  = "deploy"
)

const (
	ProjectTokenScopeReadRepository  = "read_repository"
	ProjectTokenScopeWriteRepository = "write_repository"
	ProjectTokenScopeReadPackage     = "read_package"
	ProjectTokenScopeWritePackage    = "write_package"
	ProjectTokenScopeReadAPI         = "read_api"
	ProjectTokenScopeWriteAPI        = "write_api"
)

type ProjectAccessToken struct {
	ID              int64     `dbx:"id"`
	ProjectID       int64     `dbx:"project_id"`
	Kind            string    `dbx:"kind"`
	Name            string    `dbx:"name"`
	Username        string    `dbx:"username"`
	Token           string    `dbx:"token"`
	Scopes          string    `dbx:"scopes"`
	CreatedByUserID int64     `dbx:"created_by_user_id"`
	ExpiresAt       time.Time `dbx:"expires_at"`
	RevokedAt       time.Time `dbx:"revoked_at"`
	LastUsedAt      time.Time `dbx:"last_used_at"`
	CreatedAt       time.Time `dbx:"created_at"`
	UpdatedAt       time.Time `dbx:"updated_at"`
}

func (t ProjectAccessToken) Active(now time.Time) bool {
	if !t.RevokedAt.IsZero() {
		return false
	}
	if !t.ExpiresAt.IsZero() && !t.ExpiresAt.After(now.UTC()) {
		return false
	}
	return true
}
