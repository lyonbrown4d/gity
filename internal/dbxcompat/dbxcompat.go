package dbxcompat

import (
	"context"
	"log/slog"

	collectionx "github.com/arcgolabs/collectionx/list"
	rootdbx "github.com/arcgolabs/dbx"
	columnx "github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/dialect"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/querydsl"
	schemax "github.com/arcgolabs/dbx/schema"
	"github.com/arcgolabs/dbx/schemamigrate"
)

type Assignment = querydsl.Assignment
type Column[E any, T any] = columnx.Column[E, T]
type DB = rootdbx.DB
type IDColumn[E any, T any, M idgen.Marker] = columnx.IDColumn[E, T, M]
type IDSnowflake = idgen.IDSnowflake
type OpenOption = rootdbx.OpenOption
type Option = rootdbx.Option
type Predicate = querydsl.Predicate
type Schema[E any] = schemax.Schema[E]
type SchemaDialect = dialect.Dialect
type SelectItem = querydsl.SelectItem
type SelectQuery = querydsl.SelectQuery
type Session = rootdbx.Session
type Tx = rootdbx.Tx
type Unique[E any] = schemax.Unique[E]

func AllColumns(schema schemax.Resource) *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(schema)
}

func And(predicates ...querydsl.Predicate) querydsl.Predicate {
	return querydsl.And(predicates...)
}

func AutoMigrate(ctx context.Context, session Session, schemas ...schemamigrate.Resource) (schemax.ValidationReport, error) {
	return schemamigrate.AutoMigrate(ctx, session, schemas...)
}

func ApplyOptions(opts ...Option) OpenOption {
	return rootdbx.ApplyOptions(opts...)
}

func MustSchema[S any](name string, schema S) S {
	return schemax.MustSchema(name, schema)
}

func Open(opts ...OpenOption) (*DB, error) {
	return rootdbx.Open(opts...)
}

func Select(items ...querydsl.SelectItem) *querydsl.SelectQuery {
	return querydsl.Select(items...)
}

func WithDebug(enabled bool) Option {
	return rootdbx.WithDebug(enabled)
}

func WithDialect(d dialect.Dialect) OpenOption {
	return rootdbx.WithDialect(d)
}

func WithDriver(driver string) OpenOption {
	return rootdbx.WithDriver(driver)
}

func WithDSN(dsn string) OpenOption {
	return rootdbx.WithDSN(dsn)
}

func WithLogger(logger *slog.Logger) Option {
	return rootdbx.WithLogger(logger)
}

func WithNodeID(nodeID uint16) Option {
	return rootdbx.WithNodeID(nodeID)
}
