package project

import "time"

type ProjectMember struct {
	ID        int64     `dbx:"id"`
	ProjectID int64     `dbx:"project_id"`
	UserID    int64     `dbx:"user_id"`
	Role      string    `dbx:"role"`
	CreatedAt time.Time `dbx:"created_at"`
	UpdatedAt time.Time `dbx:"updated_at"`
}
