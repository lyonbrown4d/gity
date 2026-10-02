package projectcivariable_test

import (
	"context"
	"errors"
	"log/slog"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/dialect"
	mysqlDialect "github.com/arcgolabs/dbx/dialect/mysql"
	postgresDialect "github.com/arcgolabs/dbx/dialect/postgres"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	ciports "github.com/lyonbrown4d/gity/internal/application/ports"
	"github.com/lyonbrown4d/gity/internal/infrastructure/persistence/core"
	projectcivariable "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_ci_variable"
	"github.com/samber/oops"
	_ "modernc.org/sqlite"
)

func TestRepositoryUpsertCreateAndUpdate(t *testing.T) {
	repo, recorder := newTestRepository(t)
	ctx := t.Context()

	recorder.Reset()
	created, err := repo.Upsert(ctx, projectcivariable.UpsertInput{
		ProjectID: 42,
		Key:       " deploy_token ",
		Value:     "first",
		Masked:    1,
		Protected: 0,
	})
	requireNoError(t, err)
	requireNotZero(t, created.ID)
	requireEqual(t, "DEPLOY_TOKEN", created.Key)
	requireEqual(t, "first", created.Value)
	requireEqual(t, 1, created.Masked)
	requireEqual(t, 0, created.Protected)
	requireFalse(t, created.CreatedAt.IsZero(), "created time must be set")
	requireEqual(t, created.CreatedAt, created.UpdatedAt)
	requireEqual(t, 1, recorder.TargetTableOperations())

	time.Sleep(time.Millisecond)
	recorder.Reset()
	updated, err := repo.Upsert(ctx, projectcivariable.UpsertInput{
		ProjectID: 42,
		Key:       "deploy_token",
		Value:     "second",
		Masked:    0,
		Protected: 1,
	})
	requireNoError(t, err)
	requireEqual(t, created.ID, updated.ID)
	requireEqual(t, created.CreatedAt, updated.CreatedAt)
	requireTrue(t, updated.UpdatedAt.After(created.UpdatedAt), "updated time must advance")
	requireEqual(t, "second", updated.Value)
	requireEqual(t, 0, updated.Masked)
	requireEqual(t, 1, updated.Protected)
	requireEqual(t, 1, recorder.TargetTableOperations())
}

func TestRepositoryGetByProjectAndKeyNotFound(t *testing.T) {
	repo, _ := newTestRepository(t)

	_, err := repo.GetByProjectAndKey(t.Context(), 42, "missing")

	requireErrorIs(t, err, ciports.ErrNotFound)
	oopsErr, isOops := oops.AsOops(err)
	if isOops {
		t.Fatalf("not-found unexpectedly wrapped by oops: %v", oopsErr)
	}
	requireFalse(t, isOops, "not-found is an application contract, not an infrastructure failure")
}

func TestRepositoryWrapsDatabaseErrors(t *testing.T) {
	repo, recorder := newTestRepository(t)
	databaseErr := errors.New("forced database failure")
	recorder.FailWith(databaseErr)

	_, err := repo.Upsert(t.Context(), projectcivariable.UpsertInput{ProjectID: 42, Key: "broken", Value: "value"})

	requireErrorIs(t, err, databaseErr)
	oopsErr, isOops := oops.AsOops(err)
	if isOops && oopsErr.Error() == "" {
		t.Fatal("oops error must retain a message")
	}
	requireTrue(t, isOops, "database error must use oops")
}

func TestProjectCIVariableUpsertQueryDialects(t *testing.T) {
	tests := []struct {
		name        string
		dialect     dialect.Dialect
		expectedSQL string
	}{
		{name: "sqlite", dialect: sqliteDialect.New(), expectedSQL: `ON CONFLICT ("project_id", "key") DO UPDATE SET`},
		{name: "postgres", dialect: postgresDialect.New(), expectedSQL: `ON CONFLICT ("project_id", "key") DO UPDATE SET`},
		{name: "mysql", dialect: mysqlDialect.New(), expectedSQL: "ON DUPLICATE KEY UPDATE"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			recorder := &operationRecorder{}
			databaseErr := errors.New("capture generated sql")
			recorder.FailWith(databaseErr)
			repo := newTestRepositoryWithDialect(t, test.dialect, recorder, false)

			_, err := repo.Upsert(t.Context(), projectcivariable.UpsertInput{
				ProjectID: 42,
				Key:       "DEPLOY_TOKEN",
				Value:     "secret",
				Masked:    1,
				Protected: 1,
			})

			requireErrorIs(t, err, databaseErr)
			sql := recorder.TargetTableSQL()
			requireContains(t, sql, test.expectedSQL)
			updateClause := strings.ReplaceAll(upsertUpdateClause(sql), "`", `"`)
			requireNotContains(t, updateClause, `"id" =`)
			requireNotContains(t, updateClause, `"created_at" =`)
			requireContains(t, updateClause, `"value" =`)
			requireContains(t, updateClause, `"updated_at" =`)
		})
	}
}

type operationRecorder struct {
	mu       sync.Mutex
	events   []dbx.HookEvent
	failWith error
}

func (r *operationRecorder) Before(ctx context.Context, event *dbx.HookEvent) (context.Context, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.failWith != nil && strings.Contains(event.SQL, "project_ci_variables") {
		return ctx, r.failWith
	}
	return ctx, nil
}

func (r *operationRecorder) After(_ context.Context, event *dbx.HookEvent) {
	if event == nil {
		return
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	r.events = append(r.events, *event)
}

func (r *operationRecorder) Reset() {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.events = nil
	r.failWith = nil
}

func (r *operationRecorder) FailWith(err error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.failWith = err
}

func (r *operationRecorder) TargetTableOperations() int {
	r.mu.Lock()
	defer r.mu.Unlock()
	count := 0
	for index := range r.events {
		event := &r.events[index]
		if strings.Contains(event.SQL, `"project_ci_variables"`) {
			count++
		}
	}
	return count
}

func (r *operationRecorder) TargetTableSQL() string {
	r.mu.Lock()
	defer r.mu.Unlock()
	for index := range r.events {
		event := &r.events[index]
		if strings.Contains(event.SQL, "project_ci_variables") {
			return event.SQL
		}
	}
	return ""
}

func newTestRepository(t *testing.T) (*projectcivariable.Repository, *operationRecorder) {
	t.Helper()
	recorder := &operationRecorder{}
	repo := newTestRepositoryWithDialect(t, sqliteDialect.New(), recorder, true)
	return repo, recorder
}

func newTestRepositoryWithDialect(t *testing.T, sqlDialect dialect.Dialect, recorder *operationRecorder, migrate bool) *projectcivariable.Repository {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "project-ci-variable.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN("file:"+dbPath),
		dbx.WithDialect(sqlDialect),
		dbx.ApplyOptions(
			dbx.WithHooks(recorder),
			dbx.WithLogger(slog.New(slog.DiscardHandler)),
		),
	)
	requireNoError(t, err)
	t.Cleanup(func() {
		requireNoError(t, db.Close())
	})
	if migrate {
		requireNoError(t, core.EnsureSchema(t.Context(), db))
	}
	repo, err := projectcivariable.NewRepository(db)
	requireNoError(t, err)
	return repo
}

func upsertUpdateClause(sql string) string {
	if _, clause, found := strings.Cut(sql, " DO UPDATE SET "); found {
		return clause
	}
	if _, clause, found := strings.Cut(sql, " ON DUPLICATE KEY UPDATE "); found {
		return clause
	}
	return ""
}

func requireNoError(t *testing.T, err error) {
	t.Helper()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func requireErrorIs(t *testing.T, err, target error) {
	t.Helper()
	if !errors.Is(err, target) {
		t.Fatalf("error %v does not wrap %v", err, target)
	}
}

func requireEqual[T comparable](t *testing.T, want, got T) {
	t.Helper()
	if got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
}

func requireNotZero(t *testing.T, value int64) {
	t.Helper()
	if value == 0 {
		t.Fatal("expected non-zero value")
	}
}

func requireTrue(t *testing.T, value bool, message string) {
	t.Helper()
	if !value {
		t.Fatal(message)
	}
}

func requireFalse(t *testing.T, value bool, message string) {
	t.Helper()
	if value {
		t.Fatal(message)
	}
}

func requireContains(t *testing.T, value, token string) {
	t.Helper()
	if !strings.Contains(value, token) {
		t.Fatalf("%q does not contain %q", value, token)
	}
}

func requireNotContains(t *testing.T, value, token string) {
	t.Helper()
	if strings.Contains(value, token) {
		t.Fatalf("%q unexpectedly contains %q", value, token)
	}
}
