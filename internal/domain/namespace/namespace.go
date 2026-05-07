package namespace

import "time"

type Namespace struct {
	ID          int64     `dbx:"id"`
	Kind        string    `dbx:"kind"`
	Name        string    `dbx:"name"`
	PathKey     string    `dbx:"path_key"`
	FullPath    string    `dbx:"full_path"`
	Description string    `dbx:"description"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
}
