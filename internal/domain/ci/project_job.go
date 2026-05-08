// Package ci defines CI domain models.
package ci

import "time"

type ProjectJob struct {
	ID          int64     `dbx:"id"`
	ProjectID   int64     `dbx:"project_id"`
	Kind        string    `dbx:"kind"`
	Status      string    `dbx:"status"`
	Payload     string    `dbx:"payload"`
	Result      string    `dbx:"result"`
	Attempts    int       `dbx:"attempts"`
	MaxAttempts int       `dbx:"max_attempts"`
	RunAfter    time.Time `dbx:"run_after"`
	LockedBy    string    `dbx:"locked_by"`
	LockedUntil time.Time `dbx:"locked_until"`
	LastError   string    `dbx:"last_error"`
	CreatedAt   time.Time `dbx:"created_at"`
	UpdatedAt   time.Time `dbx:"updated_at"`
	StartedAt   time.Time `dbx:"started_at"`
	FinishedAt  time.Time `dbx:"finished_at"`
}
