// Package organization defines organization domain models.
package organization

import "time"

type Organization struct {
	ID          int64     `dbx:"id"`
	Name        string    `dbx:"name"`
	PathKey     string    `dbx:"path_key"`
	FullPath    string    `dbx:"full_path"`
	Description string    `dbx:"description"`
	Visibility  string    `dbx:"visibility"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}
