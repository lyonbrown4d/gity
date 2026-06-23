package storage

import (
	"strings"

	"github.com/lyonbrown4d/gity/internal/config"
	"github.com/samber/oops"
)

// NewBackend resolves the object storage backend from application settings.
func NewBackend(settings config.Settings) (backend, error) {
	driver := strings.TrimSpace(strings.ToLower(settings.Storage.Driver))
	if driver == "" {
		driver = "local"
	}
	switch driver {
	case "local":
		return &localBackend{root: settings.Storage.Root}, nil
	case "s3":
		client, err := newS3Backend(settings)
		if err != nil {
			return nil, oops.In("storage").With("driver", driver).Wrapf(err, "create storage backend")
		}
		return client, nil
	default:
		return nil, oops.In("storage").With("driver", driver).New("unsupported storage driver")
	}
}
