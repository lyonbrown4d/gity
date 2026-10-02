package projecttodo_test

import (
	"context"
	"errors"
	"sync"
	"testing"

	"github.com/arcgolabs/dbx/dialect"
	mysqlDialect "github.com/arcgolabs/dbx/dialect/mysql"
	postgresDialect "github.com/arcgolabs/dbx/dialect/postgres"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	todoports "github.com/lyonbrown4d/gity/internal/application/ports"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
	projecttodo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_todo"
	"github.com/samber/oops"
	_ "modernc.org/sqlite"
)

func TestRepositoryCreateIfNotExistsIsIdempotent(t *testing.T) {
	repo, recorder := newTestRepository(t)
	input := projecttodo.CreateInput{
		UserID:         101,
		OrganizationID: 201,
		ProjectID:      301,
		Kind:           " issue_assigned ",
		TargetType:     " issue ",
		TargetID:       " 401 ",
		Title:          " First title ",
		Summary:        " First summary ",
		ActionURL:      " /first ",
	}

	recorder.Reset()
	created, err := repo.CreateIfNotExists(t.Context(), input)
	requireNoError(t, err)
	requireNotZero(t, created.ID)
	requireEqual(t, "issue_assigned", created.Kind)
	requireEqual(t, "issue", created.TargetType)
	requireEqual(t, "401", created.TargetID)
	requireEqual(t, "First title", created.Title)
	requireEqual(t, 1, recorder.TargetTableOperations())

	input.Title = "replacement title"
	input.Summary = "replacement summary"
	recorder.Reset()
	existing, err := repo.CreateIfNotExists(t.Context(), input)
	requireNoError(t, err)
	requireEqual(t, created.ID, existing.ID)
	requireEqual(t, created.Title, existing.Title)
	requireEqual(t, created.Summary, existing.Summary)
	requireEqual(t, created.CreatedAt, existing.CreatedAt)
	requireEqual(t, 1, recorder.TargetTableOperations())
}

func TestRepositoryCreateIfNotExistsConcurrent(t *testing.T) {
	repo, _ := newTestRepository(t)
	input := projecttodo.CreateInput{
		UserID:         101,
		OrganizationID: 201,
		ProjectID:      301,
		Kind:           tododomain.ProjectTodoKindIssueCommented,
		TargetType:     "issue_comment",
		TargetID:       "501",
		Title:          "Concurrent todo",
	}

	const workers = 16
	start := make(chan struct{})
	results := make(chan concurrentCreateResult, workers)
	var group sync.WaitGroup
	group.Add(workers)
	for range workers {
		go func() {
			defer group.Done()
			<-start
			item, err := repo.CreateIfNotExists(context.Background(), input)
			results <- concurrentCreateResult{itemID: item.ID, err: err}
		}()
	}
	close(start)
	group.Wait()
	close(results)

	var expectedID int64
	for result := range results {
		requireNoError(t, result.err)
		if expectedID == 0 {
			expectedID = result.itemID
		}
		requireEqual(t, expectedID, result.itemID)
	}
	items, err := repo.ListByUserID(t.Context(), projecttodo.ListInput{UserID: input.UserID})
	requireNoError(t, err)
	requireEqual(t, 1, items.Len())
}

func TestRepositoryCreateIfNotExistsIsolatedByUser(t *testing.T) {
	repo, _ := newTestRepository(t)
	input := projecttodo.CreateInput{
		UserID:         101,
		OrganizationID: 201,
		ProjectID:      301,
		Kind:           tododomain.ProjectTodoKindIssueCreated,
		TargetType:     "issue",
		TargetID:       "401",
		Title:          "Issue created",
	}

	first, err := repo.CreateIfNotExists(t.Context(), input)
	requireNoError(t, err)
	input.UserID = 102
	second, err := repo.CreateIfNotExists(t.Context(), input)
	requireNoError(t, err)
	if first.ID == second.ID {
		t.Fatalf("different users received the same todo ID %d", first.ID)
	}

	firstItems, err := repo.ListByUserID(t.Context(), projecttodo.ListInput{UserID: first.UserID})
	requireNoError(t, err)
	requireEqual(t, 1, firstItems.Len())
	requireEqual(t, first.ID, firstItems.Values()[0].ID)
	secondItems, err := repo.ListByUserID(t.Context(), projecttodo.ListInput{UserID: second.UserID})
	requireNoError(t, err)
	requireEqual(t, 1, secondItems.Len())
	requireEqual(t, second.ID, secondItems.Values()[0].ID)
}

func TestRepositoryMarkDoneByIDAndUser(t *testing.T) {
	repo, recorder := newTestRepository(t)
	created, err := repo.CreateIfNotExists(t.Context(), projecttodo.CreateInput{
		UserID:         101,
		OrganizationID: 201,
		ProjectID:      301,
		Kind:           tododomain.ProjectTodoKindBranchDeleted,
		TargetType:     "branch",
		TargetID:       "feature/old",
		Title:          "Branch deleted",
	})
	requireNoError(t, err)

	recorder.Reset()
	requireNoError(t, repo.MarkDoneByIDAndUser(t.Context(), created.ID, created.UserID))
	requireEqual(t, 1, recorder.TargetTableOperations())

	items, err := repo.ListByUserID(t.Context(), projecttodo.ListInput{UserID: created.UserID})
	requireNoError(t, err)
	requireEqual(t, 1, items.Len())
	marked := items.Values()[0]
	requireEqual(t, tododomain.ProjectTodoStateDone, marked.State)
	if marked.DoneAt.IsZero() {
		t.Fatal("done_at must be set")
	}
}

func TestRepositoryMarkDoneByIDAndUserNotFound(t *testing.T) {
	repo, recorder := newTestRepository(t)
	created, err := repo.CreateIfNotExists(t.Context(), projecttodo.CreateInput{
		UserID:         101,
		OrganizationID: 201,
		ProjectID:      301,
		Kind:           tododomain.ProjectTodoKindProjectCreated,
		TargetType:     "project",
		TargetID:       "301",
		Title:          "Project created",
	})
	requireNoError(t, err)

	tests := []struct {
		name   string
		id     int64
		userID int64
	}{
		{name: "other user", id: created.ID, userID: created.UserID + 1},
		{name: "missing todo", id: created.ID + 1, userID: created.UserID},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			recorder.Reset()
			markErr := repo.MarkDoneByIDAndUser(t.Context(), test.id, test.userID)
			requireErrorIs(t, markErr, todoports.ErrNotFound)
			if oopsErr, isOops := oops.AsOops(markErr); isOops {
				t.Fatalf("not-found unexpectedly wrapped by oops: %v", oopsErr)
			}
			requireEqual(t, 1, recorder.TargetTableOperations())
		})
	}

	items, err := repo.ListByUserID(t.Context(), projecttodo.ListInput{UserID: created.UserID})
	requireNoError(t, err)
	requireEqual(t, tododomain.ProjectTodoStatePending, items.Values()[0].State)
}

func TestRepositoryWrapsDatabaseErrors(t *testing.T) {
	t.Run("create", func(t *testing.T) {
		repo, recorder := newTestRepository(t)
		databaseErr := errors.New("forced create failure")
		recorder.FailWith(databaseErr)

		_, err := repo.CreateIfNotExists(t.Context(), projecttodo.CreateInput{
			UserID: 101, OrganizationID: 201, ProjectID: 301,
			Kind: "kind", TargetType: "target", TargetID: "401", Title: "title",
		})

		requireOopsError(t, err, databaseErr)
	})

	t.Run("mark done", func(t *testing.T) {
		repo, recorder := newTestRepository(t)
		created, err := repo.CreateIfNotExists(t.Context(), projecttodo.CreateInput{
			UserID: 101, OrganizationID: 201, ProjectID: 301,
			Kind: "kind", TargetType: "target", TargetID: "401", Title: "title",
		})
		requireNoError(t, err)
		databaseErr := errors.New("forced update failure")
		recorder.FailWith(databaseErr)

		err = repo.MarkDoneByIDAndUser(t.Context(), created.ID, created.UserID)

		requireOopsError(t, err, databaseErr)
	})
}

func TestRepositoryCreateIfNotExistsQueryDialects(t *testing.T) {
	tests := []struct {
		name        string
		dialect     dialect.Dialect
		expectedSQL string
	}{
		{name: "sqlite", dialect: sqliteDialect.New(), expectedSQL: `ON CONFLICT ("user_id", "kind", "target_type", "target_id") DO UPDATE SET`},
		{name: "postgres", dialect: postgresDialect.New(), expectedSQL: `ON CONFLICT ("user_id", "kind", "target_type", "target_id") DO UPDATE SET`},
		{name: "mysql", dialect: mysqlDialect.New(), expectedSQL: "ON DUPLICATE KEY UPDATE"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			recorder := &operationRecorder{}
			databaseErr := errors.New("capture generated sql")
			recorder.FailWith(databaseErr)
			repo := newTestRepositoryWithDialect(t, test.dialect, recorder, false)

			_, err := repo.CreateIfNotExists(t.Context(), projecttodo.CreateInput{
				UserID: 101, OrganizationID: 201, ProjectID: 301,
				Kind: "kind", TargetType: "target", TargetID: "401", Title: "title",
			})

			requireErrorIs(t, err, databaseErr)
			sql := recorder.TargetTableSQL()
			requireContains(t, sql, test.expectedSQL)
			updateClause := upsertUpdateClause(sql)
			requireContains(t, updateClause, "user_id")
			requireNotContains(t, updateClause, "title")
			requireNotContains(t, updateClause, "updated_at")
		})
	}
}
