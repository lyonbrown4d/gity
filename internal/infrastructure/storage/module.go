package storage

import (
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/arcgolabs/dix"
)

func NewObjectStorage(service *Service) storageports.ObjectStorage {
	return service
}

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.storage",
		dix.Description("Attachment storage runtime"),
		dix.Providers(
			dix.ProviderErr1(NewService),
			dix.Provider1(NewObjectStorage),
		),
	)
}
