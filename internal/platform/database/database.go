package database

import (
	"fmt"
	"log/slog"

	"github.com/DaiYuANg/arcgo/dbx"
	mysqlDialect "github.com/DaiYuANg/arcgo/dbx/dialect/mysql"
	postgresDialect "github.com/DaiYuANg/arcgo/dbx/dialect/postgres"
	sqliteDialect "github.com/DaiYuANg/arcgo/dbx/dialect/sqlite"
	"github.com/DaiYuANg/gity/internal/config"

	_ "github.com/go-sql-driver/mysql"
	_ "modernc.org/sqlite"
)

func NewDatabase(settings config.Settings, logger *slog.Logger) (*dbx.DB, error) {
	if settings.Database.DSN == "" {
		logger.Warn("database dsn is empty; database runtime disabled")
		return nil, nil
	}

	dialect, err := resolveDialect(settings.Database.Driver)
	if err != nil {
		return nil, err
	}

	db, err := dbx.Open(
		dbx.WithDriver(settings.Database.Driver),
		dbx.WithDSN(settings.Database.DSN),
		dbx.WithDialect(dialect),
		dbx.ApplyOptions(
			dbx.WithLogger(logger),
			dbx.WithDebug(settings.App.Environment == "development"),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("open dbx database: %w", err)
	}
	return db, nil
}

func resolveDialect(driver string) (dbx.SchemaDialect, error) {
	switch driver {
	case "sqlite":
		return sqliteDialect.New(), nil
	case "postgres":
		return postgresDialect.New(), nil
	case "mysql":
		return mysqlDialect.New(), nil
	default:
		return nil, fmt.Errorf("unsupported database driver: %s", driver)
	}
}
