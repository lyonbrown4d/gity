package core

import (
	"context"
	"database/sql"
	"strings"

	"github.com/arcgolabs/dbx"
	"github.com/samber/oops"
)

func migrationDialectName(tx *dbx.Tx) string {
	if tx == nil || tx.Dialect() == nil {
		return ""
	}
	return tx.Dialect().Name()
}

func unsupportedMigrationDialect(migrationVersion, dialectName string) error {
	return oops.In("persistence.schema").With("migration_version", migrationVersion, "dialect", dialectName).New("unsupported database dialect")
}

func mysqlIndexExists(ctx context.Context, tx *dbx.Tx, migrationVersion, tableName, indexName string) (bool, error) {
	var count int
	err := tx.QueryRowContext(ctx, `
SELECT COUNT(*)
FROM information_schema.statistics
WHERE table_schema = DATABASE()
  AND table_name = ?
  AND index_name = ?`, tableName, indexName).Scan(&count)
	if err != nil {
		return false, oops.In("persistence.schema").With("migration_version", migrationVersion, "table", tableName, "index", indexName).Wrapf(err, "check mysql index")
	}
	return count > 0, nil
}

func sqliteTableColumnExists(ctx context.Context, tx *dbx.Tx, migrationVersion, tableName, columnName string) (exists bool, err error) {
	rows, err := tx.QueryContext(ctx, `PRAGMA table_info("`+tableName+`")`)
	if err != nil {
		return false, oops.In("persistence.schema").With("migration_version", migrationVersion, "table", tableName, "column", columnName).Wrapf(err, "list sqlite table columns")
	}
	defer func() {
		if closeErr := rows.Close(); closeErr != nil && err == nil {
			err = oops.In("persistence.schema").With("migration_version", migrationVersion, "table", tableName).Wrapf(closeErr, "close sqlite column rows")
		}
	}()
	for rows.Next() {
		var cid int
		var name string
		var columnType string
		var notNull int
		var defaultValue sql.NullString
		var pk int
		if err := rows.Scan(&cid, &name, &columnType, &notNull, &defaultValue, &pk); err != nil {
			return false, oops.In("persistence.schema").With("migration_version", migrationVersion, "table", tableName, "column", columnName).Wrapf(err, "scan sqlite table column")
		}
		if strings.EqualFold(name, columnName) {
			return true, nil
		}
	}
	if err := rows.Err(); err != nil {
		return false, oops.In("persistence.schema").With("migration_version", migrationVersion, "table", tableName, "column", columnName).Wrapf(err, "iterate sqlite table columns")
	}
	return false, nil
}
