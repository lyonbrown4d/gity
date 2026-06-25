// Package database opens the configured dbx database.
package database

import (
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"

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

	dbDialect, nodeID, err := prepareDatabaseRuntime(settings)
	if err != nil {
		return nil, err
	}
	db, err := dbx.Open(
		dbx.WithDriver(settings.Database.Driver),
		dbx.WithDSN(settings.Database.DSN),
		dbx.WithDialect(dbDialect),
		dbx.ApplyOptions(
			dbx.WithLogger(logger),
			dbx.WithDebug(settings.App.Environment == "development"),
			dbx.WithNodeID(nodeID),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("open dbx database: %w", err)
	}
	if err := configureDatabaseRuntime(settings.Database.Driver, db); err != nil {
		return nil, err
	}
	return db, nil
}

func prepareDatabaseRuntime(settings config.Settings) (dialect.Dialect, uint16, error) {
	if settings.Database.Driver == "sqlite" {
		if err := ensureSQLiteDatabaseDir(settings.Database.DSN); err != nil {
			return nil, 0, oops.In("database").Wrapf(err, "prepare sqlite database directory")
		}
	}
	nodeID, err := databaseNodeID(settings.Database.NodeID)
	if err != nil {
		return nil, 0, err
	}
	dbDialect, err := resolveDialect(settings.Database.Driver)
	if err != nil {
		return nil, 0, err
	}
	return dbDialect, nodeID, nil
}

func databaseNodeID(value int) (uint16, error) {
	if value < 1 || value > 1023 {
		return 0, fmt.Errorf("database node id %d out of range [1,1023]", value)
	}
	return uint16(value), nil
}

func configureDatabaseRuntime(driver string, db *dbx.DB) error {
	if driver != "sqlite" {
		return nil
	}
	return configureSQLiteRuntime(db)
}

func configureSQLiteRuntime(db *dbx.DB) error {
	if db == nil || db.SQLDB() == nil {
		return nil
	}
	sqlDB := db.SQLDB()
	sqlDB.SetMaxOpenConns(1)
	sqlDB.SetMaxIdleConns(1)
	if _, err := sqlDB.Exec("PRAGMA busy_timeout = 5000"); err != nil {
		return oops.In("database").Wrapf(err, "configure sqlite busy timeout")
	}
	return nil
}

func ensureSQLiteDatabaseDir(dsn string) error {
	const sqlitePrefix = "file:"
	if !strings.HasPrefix(dsn, sqlitePrefix) {
		return nil
	}
	trimmed, _ := strings.CutPrefix(dsn, sqlitePrefix)
	if isSQLiteMemoryDSN(trimmed) {
		return nil
	}

	dbPath := strings.Split(trimmed, "?")[0]
	dbPath, _ = strings.CutPrefix(dbPath, "//")
	if dbPath == "" || isSQLiteRootPath(dbPath) {
		return nil
	}

	if err := os.MkdirAll(filepath.Dir(filepath.Clean(dbPath)), 0o750); err != nil {
		return oops.In("database").Wrapf(err, "create sqlite data directory")
	}
	return nil
}

func isSQLiteMemoryDSN(dsn string) bool {
	return dsn == "" || dsn == ":memory:" || strings.HasPrefix(dsn, ":memory:")
}

func isSQLiteRootPath(dbPath string) bool {
	return dbPath == ":memory:" || (strings.HasPrefix(dbPath, "/") && strings.Count(dbPath, "/") == 1)
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
