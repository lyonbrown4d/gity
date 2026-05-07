package lfs

import "time"

type ProjectLFSObject struct {
	ID         int64     `dbx:"id"`
	ProjectID  int64     `dbx:"project_id"`
	OID        string    `dbx:"oid"`
	ByteSize   int64     `dbx:"byte_size"`
	StorageKey string    `dbx:"storage_key"`
	CreatedAt  time.Time `dbx:"created_at"`
	UpdatedAt  time.Time `dbx:"updated_at"`
}
