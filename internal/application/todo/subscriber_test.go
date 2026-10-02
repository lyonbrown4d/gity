package todo_test

import (
	"context"
	"slices"
	"testing"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/eventx"
	appports "github.com/lyonbrown4d/gity/internal/application/ports"
	todoservice "github.com/lyonbrown4d/gity/internal/application/todo"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
)

func TestSubscriberCreatesIssueCreatedTodosForProjectAndOrganizationMembers(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	todos := newMemoryTodoRepository()
	bus := newTestBus(t, todos)

	publishEvent(ctx, t, bus, issuedomain.ProjectIssueCreated{
		ProjectID:    10,
		IssueID:      100,
		IssueIID:     7,
		AuthorUserID: 1,
		Title:        "Production incident follow-up",
		State:        "opened",
	})

	assertTodoUsers(t, todos.items, []int64{2, 3, 4})
	for index := range todos.items {
		item := todos.items[index]
		if item.Kind != tododomain.ProjectTodoKindIssueCreated {
			t.Fatalf("expected issue created kind, got %q", item.Kind)
		}
		if item.ActionURL != "/app/projects/20/10/issues/7" {
			t.Fatalf("expected issue action url, got %q", item.ActionURL)
		}
	}
}

func TestSubscriberCreatesIssueAssignedTodoForAssigneeOnly(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	todos := newMemoryTodoRepository()
	bus := newTestBus(t, todos)

	publishEvent(ctx, t, bus, issuedomain.ProjectIssueAssigned{
		ProjectID:      10,
		IssueID:        101,
		IssueIID:       8,
		AuthorUserID:   1,
		AssigneeUserID: 2,
		ActorUserID:    3,
		Title:          "Triage package publish failure",
	})
	assertTodoUsers(t, todos.items, []int64{2})
	if todos.items[0].Kind != tododomain.ProjectTodoKindIssueAssigned {
		t.Fatalf("expected issue assigned kind, got %q", todos.items[0].Kind)
	}

	before := len(todos.items)
	publishEvent(ctx, t, bus, issuedomain.ProjectIssueAssigned{
		ProjectID:      10,
		IssueID:        102,
		IssueIID:       9,
		AssigneeUserID: 4,
		ActorUserID:    4,
		Title:          "Self assignment should not notify",
	})
	if len(todos.items) != before {
		t.Fatalf("expected self assignment to create no todo, got %d new items", len(todos.items)-before)
	}
}

func TestSubscriberCreatesMergeRequestMergedTodoForAuthor(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	todos := newMemoryTodoRepository()
	bus := newTestBus(t, todos)

	publishEvent(ctx, t, bus, mergedomain.ProjectMergeRequestMerged{
		ProjectID:      10,
		MergeRequestID: 200,
		MergeIID:       3,
		ActorUserID:    2,
		AuthorUserID:   1,
		Title:          "Add GitHub Actions compatibility layer",
		TargetBranch:   "main",
	})
	assertTodoUsers(t, todos.items, []int64{1})
	if todos.items[0].Kind != tododomain.ProjectTodoKindMergeRequestMerged {
		t.Fatalf("expected merge request merged kind, got %q", todos.items[0].Kind)
	}
}

func newTestBus(t *testing.T, todoRepo *memoryTodoRepository) *eventx.Bus {
	t.Helper()
	bus := eventx.New(eventx.WithParallelDispatch(false))
	service := todoservice.NewService(todoRepo)
	subscriber := todoservice.NewSubscriber(service, memoryProjectRepository{}, memoryProjectMemberRepository{}, memoryOrganizationMemberRepository{})
	if err := subscriber.Subscribe(bus); err != nil {
		t.Fatalf("subscribe todo events: %v", err)
	}
	t.Cleanup(subscriber.Close)
	return bus
}

func publishEvent(ctx context.Context, t *testing.T, bus *eventx.Bus, event eventx.Event) {
	t.Helper()
	if err := bus.Publish(ctx, event); err != nil {
		t.Fatalf("publish event %s: %v", event.Name(), err)
	}
}

func assertTodoUsers(t *testing.T, items []tododomain.ProjectTodo, want []int64) {
	t.Helper()
	got := make([]int64, 0, len(items))
	for index := range items {
		got = append(got, items[index].UserID)
	}
	slices.Sort(got)
	slices.Sort(want)
	if !slices.Equal(got, want) {
		t.Fatalf("expected todo users %v, got %v", want, got)
	}
}

type memoryTodoRepository struct {
	items []tododomain.ProjectTodo
	next  int64
}

func newMemoryTodoRepository() *memoryTodoRepository {
	return &memoryTodoRepository{next: 1}
}

func (r *memoryTodoRepository) CreateIfNotExists(_ context.Context, input appports.CreateProjectTodoInput) (tododomain.ProjectTodo, error) {
	for index := range r.items {
		item := r.items[index]
		if item.UserID == input.UserID && item.Kind == input.Kind && item.TargetType == input.TargetType && item.TargetID == input.TargetID {
			return item, nil
		}
	}
	now := time.Now().UTC()
	item := tododomain.ProjectTodo{
		ID:             r.next,
		UserID:         input.UserID,
		OrganizationID: input.OrganizationID,
		ProjectID:      input.ProjectID,
		Kind:           input.Kind,
		State:          tododomain.ProjectTodoStatePending,
		TargetType:     input.TargetType,
		TargetID:       input.TargetID,
		Title:          input.Title,
		Summary:        input.Summary,
		ActionURL:      input.ActionURL,
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	r.next++
	r.items = append(r.items, item)
	return item, nil
}

func (r *memoryTodoRepository) ListByUserID(_ context.Context, input appports.ListProjectTodosInput) (*collectionx.List[tododomain.ProjectTodo], error) {
	items := make([]tododomain.ProjectTodo, 0, len(r.items))
	for index := range r.items {
		item := r.items[index]
		if item.UserID == input.UserID && (input.State == "" || item.State == input.State) {
			items = append(items, item)
		}
	}
	return collectionx.NewList(items...), nil
}

func (r *memoryTodoRepository) MarkDoneByIDAndUser(_ context.Context, id, userID int64) error {
	for index := range r.items {
		if r.items[index].ID == id && r.items[index].UserID == userID {
			r.items[index].State = tododomain.ProjectTodoStateDone
			return nil
		}
	}
	return appports.ErrNotFound
}

type memoryProjectRepository struct{}

func (memoryProjectRepository) List(context.Context, *int64) (*collectionx.List[projectdomain.Project], error) {
	return collectionx.NewList[projectdomain.Project](), nil
}

func (memoryProjectRepository) GetByID(_ context.Context, id int64) (projectdomain.Project, error) {
	return testProject(id), nil
}

func (memoryProjectRepository) GetIncludingDeletedByID(_ context.Context, id int64) (projectdomain.Project, error) {
	return testProject(id), nil
}

func (memoryProjectRepository) GetByFullPath(context.Context, string) (projectdomain.Project, error) {
	return projectdomain.Project{}, appports.ErrNotFound
}

func (memoryProjectRepository) Create(context.Context, appports.CreateProjectInput, organizationdomain.Organization) (projectdomain.Project, error) {
	return projectdomain.Project{}, nil
}

func (memoryProjectRepository) MarkPendingDeleteByID(context.Context, int64, time.Time) error {
	return nil
}

func (memoryProjectRepository) DeleteByID(context.Context, int64) error {
	return nil
}

type memoryProjectMemberRepository struct{}

func (memoryProjectMemberRepository) ListByProjectID(context.Context, int64) (*collectionx.List[projectdomain.ProjectMember], error) {
	return collectionx.NewList(
		projectdomain.ProjectMember{ProjectID: 10, UserID: 3, Role: "developer"},
		projectdomain.ProjectMember{ProjectID: 10, UserID: 4, Role: "maintainer"},
	), nil
}

func (memoryProjectMemberRepository) FindByProjectAndUser(context.Context, int64, int64) (projectdomain.ProjectMember, error) {
	return projectdomain.ProjectMember{}, appports.ErrNotFound
}

func (memoryProjectMemberRepository) Create(context.Context, appports.CreateProjectMemberInput) (projectdomain.ProjectMember, error) {
	return projectdomain.ProjectMember{}, nil
}

func (memoryProjectMemberRepository) UpdateRoleByID(context.Context, int64, string) error {
	return nil
}

func (memoryProjectMemberRepository) DeleteByProjectAndUser(context.Context, int64, int64) error {
	return nil
}

type memoryOrganizationMemberRepository struct{}

func (memoryOrganizationMemberRepository) ListByOrganizationID(context.Context, int64) (*collectionx.List[organizationdomain.OrganizationMember], error) {
	return collectionx.NewList(
		organizationdomain.OrganizationMember{OrganizationID: 20, UserID: 1, Role: "owner"},
		organizationdomain.OrganizationMember{OrganizationID: 20, UserID: 2, Role: "developer"},
		organizationdomain.OrganizationMember{OrganizationID: 20, UserID: 3, Role: "maintainer"},
	), nil
}

func (memoryOrganizationMemberRepository) FindByOrganizationAndUser(context.Context, int64, int64) (organizationdomain.OrganizationMember, error) {
	return organizationdomain.OrganizationMember{}, appports.ErrNotFound
}

func (memoryOrganizationMemberRepository) Create(context.Context, appports.CreateOrganizationMemberInput) (organizationdomain.OrganizationMember, error) {
	return organizationdomain.OrganizationMember{}, nil
}

func testProject(id int64) projectdomain.Project {
	return projectdomain.Project{ID: id, OrganizationID: 20, Name: "Gity", PathKey: "gity", FullPath: "core/gity", Visibility: "private", DefaultBranch: "main", Status: "active"}
}

var _ appports.ProjectTodoRepository = (*memoryTodoRepository)(nil)
var _ appports.ProjectRepository = memoryProjectRepository{}
var _ appports.ProjectMemberRepository = memoryProjectMemberRepository{}
var _ appports.OrganizationMemberRepository = memoryOrganizationMemberRepository{}
