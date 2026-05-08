// Package event defines domain event contracts.
package event

import "time"

type Event interface {
	Name() string
}

type Metadata struct {
	OccurredAt time.Time `json:"occurred_at"`
}

func NewMetadata() Metadata {
	return Metadata{OccurredAt: time.Now().UTC()}
}
