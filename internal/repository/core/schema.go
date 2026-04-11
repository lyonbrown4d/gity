package core

import (
	"context"

	"github.com/DaiYuANg/arcgo/dbx"
	"github.com/DaiYuANg/gity/internal/entity"
)

func EnsureSchema(ctx context.Context, db *dbx.DB) error {
	if db == nil {
		return nil
	}
	_, err := dbx.AutoMigrate(ctx, db, entity.UserSchema, entity.NamespaceSchema, entity.ProjectSchema, entity.NamespaceMemberSchema)
	return err
}
