// Package database opens the configured dbx database.
package database

import (
	"fmt"
	"log/slog"

	"github.com/lyonbrown4d/gity/internal/config"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/dialect"
	mysqlDialect "github.com/arcgolabs/dbx/dialect/mysql"
	postgresDialect "github.com/arcgolabs/dbx/dialect/postgres"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "github.com/go-sql-driver/mysql" // Register the MySQL database driver.
	_ "github.com/lib/pq"              // Register the PostgreSQL database driver.
	"github.com/samber/oops"
	_ "modernc.org/sqlite" // Register the SQLite database driver.
)

func NewDatabase(settings config.Settings, logger *slog.Logger) (*dbx.DB, error) {
	if settings.Database.DSN == "" {
		logger.Warn("database dsn is empty; database runtime disabled")
		return nil, oops.In("database").New("database dsn is required")
	}

	dbDialect, err := resolveDialect(settings.Database.Driver)
	if err != nil {
		return nil, err
	}
	if settings.Database.NodeID < 1 || settings.Database.NodeID > 1023 {
		return nil, fmt.Errorf("database node id %d out of range [1,1023]", settings.Database.NodeID)
	}

	db, err := dbx.Open(
		dbx.WithDriver(settings.Database.Driver),
		dbx.WithDSN(settings.Database.DSN),
		dbx.WithDialect(dbDialect),
		dbx.ApplyOptions(
			dbx.WithLogger(logger),
			dbx.WithDebug(settings.App.Environment == "development"),
			dbx.WithNodeID(uint16(settings.Database.NodeID)),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("open dbx database: %w", err)
	}
	return db, nil
}

func resolveDialect(driver string) (dialect.Dialect, error) {
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
