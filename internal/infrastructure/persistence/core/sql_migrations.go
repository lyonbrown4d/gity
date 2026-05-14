package core

import (
	"context"
	"embed"
	"io/fs"
	"path"
	"strings"

	"github.com/arcgolabs/dbx"
	dbxmigrate "github.com/arcgolabs/dbx/migrate"
	"github.com/samber/oops"
)

const sqlMigrationHistoryTable = "schema_history"

//go:embed migrations/mysql/*.sql migrations/postgres/*.sql migrations/sqlite/*.sql
var sqlMigrationFS embed.FS

func ensureSQLMigrations(ctx context.Context, db *dbx.DB) error {
	if db == nil {
		return nil
	}
	if db.SQLDB() == nil || db.Dialect() == nil {
		return oops.In("persistence.schema").New("sql migration database is not ready")
	}

	dialectName := strings.TrimSpace(db.Dialect().Name())
	sourceDir := path.Join("migrations", dialectName)
	if _, err := fs.ReadDir(sqlMigrationFS, sourceDir); err != nil {
		return oops.In("persistence.schema").With("dialect", dialectName, "dir", sourceDir).Wrapf(err, "load sql migration directory")
	}

	runner := dbxmigrate.NewRunner(db.SQLDB(), db.Dialect(), dbxmigrate.RunnerOptions{
		HistoryTable:    sqlMigrationHistoryTable,
		AllowOutOfOrder: true,
		ValidateHash:    true,
	})
	if _, err := runner.UpSQL(ctx, dbxmigrate.FileSource{FS: sqlMigrationFS, Dir: sourceDir}); err != nil {
		return oops.In("persistence.schema").With("dialect", dialectName).Wrapf(err, "apply sql migrations")
	}
	return nil
}
