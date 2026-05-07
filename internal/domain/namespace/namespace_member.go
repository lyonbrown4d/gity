package namespace

import "time"

type NamespaceMember struct {
	ID          int64     `dbx:"id"`
	NamespaceID int64     `dbx:"namespace_id"`
	UserID      int64     `dbx:"user_id"`
	Role        string    `dbx:"role"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}
