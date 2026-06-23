// Package storage wires object storage services.
package storage

import (
	"github.com/arcgolabs/dix"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
)

func NewObjectStorage(service *Service) storageports.ObjectStorage {
	return service
}

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.storage",
		dix.Description("Attachment storage runtime"),
		dix.Providers(
			dix.ProviderErr1(NewBackend),
			dix.Provider1(NewServiceWithBackend),
			dix.Provider1(NewObjectStorage),
		),
	)
}
