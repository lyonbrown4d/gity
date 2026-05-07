package persistence

import (
	"errors"

	appports "github.com/DaiYuANg/gity/internal/application/ports"
	collectionx "github.com/arcgolabs/collectionx/list"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

func NormalizeError(err error) error {
	if err == nil {
		return nil
	}
	if IsNotFound(err) {
		return appports.ErrNotFound
	}
	return err
}

func IsNotFound(err error) bool {
	return errors.Is(err, appports.ErrNotFound) || errors.Is(err, dbxrepo.ErrNotFound)
}

func One[T any](item T, err error) (T, error) {
	return item, NormalizeError(err)
}

func Many[T any](items *collectionx.List[T], err error) (*collectionx.List[T], error) {
	return items, NormalizeError(err)
}
