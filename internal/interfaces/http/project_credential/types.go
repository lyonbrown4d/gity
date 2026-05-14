package projectcredential

type projectCredentialInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type createProjectTokenInput struct {
	ProjectID     int64           `path:"id"`
	Authorization string          `header:"Authorization"`
	Body          createTokenBody `json:"body"`
}

type projectTokenInput struct {
	ProjectID     int64  `path:"id"`
	TokenID       int64  `path:"token_id"`
	Authorization string `header:"Authorization"`
}

type createDeployKeyInput struct {
	ProjectID     int64               `path:"id"`
	Authorization string              `header:"Authorization"`
	Body          createDeployKeyBody `json:"body"`
}

type deployKeyInput struct {
	ProjectID     int64  `path:"id"`
	KeyID         int64  `path:"key_id"`
	Authorization string `header:"Authorization"`
}

type credentialOutput struct {
	Body any `json:"body"`
}

type createTokenBody struct {
	Name            string   `json:"name"`
	Username        string   `json:"username"`
	Scopes          []string `json:"scopes"`
	ExpiresAt       string   `json:"expires_at"`
	CreatedByUserID int64    `json:"created_by_user_id"`
}

type createDeployKeyBody struct {
	Title           string `json:"title"`
	PublicKey       string `json:"public_key"`
	CanPush         bool   `json:"can_push"`
	CreatedByUserID int64  `json:"created_by_user_id"`
}

type projectTokenView struct {
	ID              string   `json:"id"`
	ProjectID       string   `json:"project_id"`
	Kind            string   `json:"kind"`
	Name            string   `json:"name"`
	Username        string   `json:"username"`
	Scopes          []string `json:"scopes"`
	CreatedByUserID string   `json:"created_by_user_id"`
	ExpiresAt       string   `json:"expires_at"`
	RevokedAt       string   `json:"revoked_at"`
	LastUsedAt      string   `json:"last_used_at"`
	Active          bool     `json:"active"`
	CreatedAt       string   `json:"created_at"`
	UpdatedAt       string   `json:"updated_at"`
}

type createdProjectTokenView struct {
	ProjectToken projectTokenView `json:"project_token"`
	Token        string           `json:"token"`
}

type deployKeyView struct {
	ID              string `json:"id"`
	ProjectID       string `json:"project_id"`
	Title           string `json:"title"`
	Fingerprint     string `json:"fingerprint"`
	PublicKey       string `json:"public_key"`
	CanPush         bool   `json:"can_push"`
	CreatedByUserID string `json:"created_by_user_id"`
	LastUsedAt      string `json:"last_used_at"`
	CreatedAt       string `json:"created_at"`
	UpdatedAt       string `json:"updated_at"`
}

func (in projectCredentialInput) AuthorizationHeader() string { return in.Authorization }
func (in projectCredentialInput) ProjectIDValue() int64       { return in.ProjectID }
func (in createProjectTokenInput) AuthorizationHeader() string {
	return in.Authorization
}
func (in createProjectTokenInput) ProjectIDValue() int64 { return in.ProjectID }
func (in projectTokenInput) AuthorizationHeader() string { return in.Authorization }
func (in projectTokenInput) ProjectIDValue() int64       { return in.ProjectID }
func (in createDeployKeyInput) AuthorizationHeader() string {
	return in.Authorization
}
func (in createDeployKeyInput) ProjectIDValue() int64 { return in.ProjectID }
func (in deployKeyInput) AuthorizationHeader() string { return in.Authorization }
func (in deployKeyInput) ProjectIDValue() int64       { return in.ProjectID }
