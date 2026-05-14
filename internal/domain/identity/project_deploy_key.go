package identity

import "time"

type ProjectDeployKey struct {
	ID              int64     `dbx:"id"`
	ProjectID       int64     `dbx:"project_id"`
	Title           string    `dbx:"title"`
	Fingerprint     string    `dbx:"fingerprint"`
	PublicKey       string    `dbx:"public_key"`
	CanPush         int       `dbx:"can_push"`
	CreatedByUserID int64     `dbx:"created_by_user_id"`
	LastUsedAt      time.Time `dbx:"last_used_at"`
	CreatedAt       time.Time `dbx:"created_at"`
	UpdatedAt       time.Time `dbx:"updated_at"`
}

func (k ProjectDeployKey) PushEnabled() bool {
	return k.CanPush != 0
}
