// Package projectcounter allocates per-project internal IDs for GitLab-like resources.
package projectcounter

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/arcgolabs/dbx"
	"github.com/samber/oops"
)

const (
	CounterIssue        = "issue"
	CounterMergeRequest = "merge_request"
	CounterPipeline     = "pipeline"
)

func Next(ctx context.Context, tx *dbx.Tx, projectID int64, counterName string) (int64, error) {
	counterName = strings.TrimSpace(counterName)
	if tx == nil || tx.Dialect() == nil {
		return 0, oops.In("persistence.project_counter").New("transaction with dialect is required")
	}
	if projectID <= 0 || counterName == "" {
		return 0, oops.In("persistence.project_counter").With("project_id", projectID, "counter_name", counterName).New("invalid project counter input")
	}

	switch tx.Dialect().Name() {
	case "sqlite", "postgres":
		return nextReturning(ctx, tx, projectID, counterName)
	case "mysql":
		return nextMySQL(ctx, tx, projectID, counterName)
	default:
		return 0, oops.In("persistence.project_counter").With("dialect", tx.Dialect().Name()).New("unsupported project counter dialect")
	}
}

func nextReturning(ctx context.Context, tx *dbx.Tx, projectID int64, counterName string) (int64, error) {
	dialect := tx.Dialect()
	q := dialect.QuoteIdent
	table := q("project_iid_counters")
	now := time.Now().UTC()
	statement := fmt.Sprintf(
		`INSERT INTO %s (%s, %s, %s, %s, %s) VALUES (%s, %s, 1, %s, %s) ON CONFLICT (%s, %s) DO UPDATE SET %s = %s.%s + 1, %s = excluded.%s RETURNING %s`,
		table,
		q("project_id"), q("counter_name"), q("current_value"), q("created_at"), q("updated_at"),
		dialect.BindVar(1), dialect.BindVar(2), dialect.BindVar(3), dialect.BindVar(4),
		q("project_id"), q("counter_name"),
		q("current_value"), table, q("current_value"),
		q("updated_at"), q("updated_at"),
		q("current_value"),
	)
	var next int64
	if err := tx.QueryRowContext(ctx, statement, projectID, counterName, now, now).Scan(&next); err != nil {
		return 0, oops.In("persistence.project_counter").With("project_id", projectID, "counter_name", counterName).Wrapf(err, "advance project counter")
	}
	return next, nil
}

func nextMySQL(ctx context.Context, tx *dbx.Tx, projectID int64, counterName string) (int64, error) {
	q := tx.Dialect().QuoteIdent
	now := time.Now().UTC()
	statement := fmt.Sprintf(
		"INSERT INTO %s (%s, %s, %s, %s, %s) VALUES (?, ?, LAST_INSERT_ID(1), ?, ?) ON DUPLICATE KEY UPDATE %s = LAST_INSERT_ID(%s + 1), %s = VALUES(%s)",
		q("project_iid_counters"),
		q("project_id"), q("counter_name"), q("current_value"), q("created_at"), q("updated_at"),
		q("current_value"), q("current_value"),
		q("updated_at"), q("updated_at"),
	)
	if _, err := tx.ExecContext(ctx, statement, projectID, counterName, now, now); err != nil {
		return 0, oops.In("persistence.project_counter").With("project_id", projectID, "counter_name", counterName).Wrapf(err, "advance mysql project counter")
	}
	var next int64
	if err := tx.QueryRowContext(ctx, "SELECT LAST_INSERT_ID()").Scan(&next); err != nil {
		return 0, oops.In("persistence.project_counter").With("project_id", projectID, "counter_name", counterName).Wrapf(err, "read mysql project counter")
	}
	return next, nil
}
