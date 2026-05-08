package ci

import "time"

type ProjectRunner struct {
	ID            int64     `dbx:"id"              json:"id"`
	ProjectID     int64     `dbx:"project_id"      json:"project_id"`
	Name          string    `dbx:"name"            json:"name"`
	Description   string    `dbx:"description"     json:"description"`
	Tags          string    `dbx:"tags"            json:"tags"`
	Token         string    `dbx:"token"           json:"token"`
	Status        string    `dbx:"status"          json:"status"`
	Active        int       `dbx:"active"          json:"active"`
	LastContactAt time.Time `dbx:"last_contact_at" json:"last_contact_at"`
	CreatedAt     time.Time `dbx:"created_at"      json:"created_at"`
	UpdatedAt     time.Time `dbx:"updated_at"      json:"updated_at"`
}
