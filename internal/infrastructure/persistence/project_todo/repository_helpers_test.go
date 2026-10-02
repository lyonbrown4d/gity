package projecttodo_test

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"path/filepath"
	"strings"
	"sync"
	"testing"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/dialect"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	"github.com/lyonbrown4d/gity/internal/infrastructure/persistence/core"
	projecttodo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_todo"
	"github.com/samber/oops"
)

type concurrentCreateResult struct {
	itemID int64
	err    error
}

type operationRecorder struct {
	mu       sync.Mutex
	events   []dbx.HookEvent
	failWith error
}

func (r *operationRecorder) Before(ctx context.Context, event *dbx.HookEvent) (context.Context, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.failWith != nil && event != nil && strings.Contains(event.SQL, "project_todos") {
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
		if strings.Contains(r.events[index].SQL, "project_todos") {
			count++
		}
	}
	return count
}

func (r *operationRecorder) TargetTableSQL() string {
	r.mu.Lock()
	defer r.mu.Unlock()
	for index := range r.events {
		if strings.Contains(r.events[index].SQL, "project_todos") {
			return r.events[index].SQL
		}
	}
	return ""
}

func newTestRepository(t *testing.T) (*projecttodo.Repository, *operationRecorder) {
	t.Helper()
	recorder := &operationRecorder{}
	repo := newTestRepositoryWithDialect(t, sqliteDialect.New(), recorder, true)
	return repo, recorder
}

func newTestRepositoryWithDialect(t *testing.T, sqlDialect dialect.Dialect, recorder *operationRecorder, migrate bool) *projecttodo.Repository {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "project-todo.db")
	dsn := fmt.Sprintf("file:%s?_pragma=busy_timeout(5000)&_pragma=journal_mode(WAL)", dbPath)
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(dsn),
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
	repo, err := projecttodo.NewRepository(db)
	requireNoError(t, err)
	return repo
}

func upsertUpdateClause(sql string) string {
	if _, clause, found := strings.Cut(sql, " DO UPDATE SET "); found {
		clause, _, _ = strings.Cut(clause, " RETURNING ")
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

func requireOopsError(t *testing.T, err, target error) {
	t.Helper()
	requireErrorIs(t, err, target)
	oopsErr, isOops := oops.AsOops(err)
	if !isOops {
		t.Fatalf("error is not wrapped with oops: %v", err)
	}
	if oopsErr.Error() == "" {
		t.Fatal("oops error must retain a message")
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
