package organization

import "time"

type OrganizationMember struct {
	ID             int64     `dbx:"id"`
	OrganizationID int64     `dbx:"organization_id"`
	UserID         int64     `dbx:"user_id"`
	Role           string    `dbx:"role"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}
