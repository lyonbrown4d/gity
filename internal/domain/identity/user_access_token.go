package identity

import "time"

type UserAccessToken struct {
	ID        int64     `dbx:"id"`
	UserID    int64     `dbx:"user_id"`
	Name      string    `dbx:"name"`
	Token     string    `dbx:"token"`
	CreatedAt time.Time `dbx:"created_at"`
	UpdatedAt time.Time `dbx:"updated_at"`
}
