// Package identity defines identity domain models.
package identity

import "time"

type User struct {
	ID          int64     `dbx:"id"`
	Username    string    `dbx:"username"`
	DisplayName string    `dbx:"display_name"`
	Email       string    `dbx:"email"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}
