package todo

import (
	"context"
	"fmt"

	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/eventx"
	todoports "github.com/lyonbrown4d/gity/internal/application/ports"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	tododomain "github.com/lyonbrown4d/gity/internal/domain/todo"
	"github.com/samber/oops"
)

type Subscriber struct {
	service                *Service
	projectRepo            todoports.ProjectRepository
	projectMemberRepo      todoports.ProjectMemberRepository
	organizationMemberRepo todoports.OrganizationMemberRepository
	unsubscribe            []func()
}

func NewSubscriber(service *Service, projectRepo todoports.ProjectRepository, projectMemberRepo todoports.ProjectMemberRepository, organizationMemberRepo todoports.OrganizationMemberRepository) *Subscriber {
	return &Subscriber{service: service, projectRepo: projectRepo, projectMemberRepo: projectMemberRepo, organizationMemberRepo: organizationMemberRepo}
}

func (s *Subscriber) Subscribe(bus *eventx.Bus) error {
	if bus == nil || s == nil {
		return nil
	}
	return oops.Join(
		subscribeTodoEvent(s, bus, s.handleProjectCreated),
		subscribeTodoEvent(s, bus, s.handleProjectBranchProtectionChanged),
		subscribeTodoEvent(s, bus, s.handleProjectBranchDeleted),
		subscribeTodoEvent(s, bus, s.handleProjectMergeRequestMerged),
		subscribeTodoEvent(s, bus, s.handleProjectIssueCreated),
		subscribeTodoEvent(s, bus, s.handleProjectIssueAssigned),
		subscribeTodoEvent(s, bus, s.handleProjectIssueCommented),
	)
}

func (s *Subscriber) Close() {
	for _, unsubscribe := range s.unsubscribe {
		if unsubscribe != nil {
			unsubscribe()
		}
	}
	s.unsubscribe = nil
}

func subscribeTodoEvent[T eventx.Event](subscriber *Subscriber, bus *eventx.Bus, handler func(context.Context, T) error) error {
	unsubscribe, err := bus.Subscribe(handler)
	if err != nil {
		return oops.In("todo").Wrapf(err, "subscribe todo event")
	}
	subscriber.unsubscribe = append(subscriber.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) handleProjectCreated(ctx context.Context, event projectdomain.ProjectCreated) error {
	recipients, err := s.organizationRecipients(ctx, event.OrganizationID)
	if err != nil {
		return err
	}
	return s.createForRecipients(ctx, recipients, todoports.CreateProjectTodoInput{
		OrganizationID: event.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindProjectCreated,
		TargetType:     "project",
		TargetID:       formatID(event.ProjectID),
		Title:          "Project created: " + event.FullPath,
		Summary:        "A new project is ready for collaboration.",
		ActionURL:      projectURL(event.OrganizationID, event.ProjectID),
	})
}

func (s *Subscriber) handleProjectBranchProtectionChanged(ctx context.Context, event projectdomain.ProjectBranchProtectionChanged) error {
	project, recipients, err := s.projectRecipients(ctx, event.ProjectID)
	if err != nil {
		return err
	}
	action := "updated"
	if !event.Protected {
		action = "removed"
	}
	return s.createForRecipients(ctx, recipients, todoports.CreateProjectTodoInput{
		OrganizationID: project.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindBranchProtectionChanged,
		TargetType:     "branch_protection",
		TargetID:       event.BranchName,
		Title:          fmt.Sprintf("Branch protection %s: %s", action, event.BranchName),
		Summary:        "Review protected branch rules before pushing or merging.",
		ActionURL:      projectURL(project.OrganizationID, project.ID) + "?tab=settings",
	})
}

func (s *Subscriber) handleProjectBranchDeleted(ctx context.Context, event projectdomain.ProjectBranchDeleted) error {
	project, recipients, err := s.projectRecipients(ctx, event.ProjectID)
	if err != nil {
		return err
	}
	return s.createForRecipients(ctx, recipients, todoports.CreateProjectTodoInput{
		OrganizationID: project.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindBranchDeleted,
		TargetType:     "branch",
		TargetID:       event.BranchName,
		Title:          "Branch deleted: " + event.BranchName,
		Summary:        "A branch was deleted from this project.",
		ActionURL:      projectURL(project.OrganizationID, project.ID) + "?tab=branches",
	})
}

func (s *Subscriber) handleProjectMergeRequestMerged(ctx context.Context, event mergedomain.ProjectMergeRequestMerged) error {
	if event.AuthorUserID <= 0 || event.AuthorUserID == event.ActorUserID {
		return nil
	}
	project, err := s.projectRepo.GetIncludingDeletedByID(ctx, event.ProjectID)
	if err != nil {
		return oops.In("todo").With("project_id", event.ProjectID, "merge_request_id", event.MergeRequestID).Wrapf(err, "load project for merged merge request todo")
	}
	return s.createForRecipients(ctx, []int64{event.AuthorUserID}, todoports.CreateProjectTodoInput{
		OrganizationID: project.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindMergeRequestMerged,
		TargetType:     "merge_request",
		TargetID:       formatID(event.MergeRequestID),
		Title:          fmt.Sprintf("Merge request !%d merged", event.MergeIID),
		Summary:        event.Title,
		ActionURL:      projectURL(project.OrganizationID, project.ID) + "?tab=merge-requests",
	})
}

func (s *Subscriber) handleProjectIssueCreated(ctx context.Context, event issuedomain.ProjectIssueCreated) error {
	project, recipients, err := s.projectRecipients(ctx, event.ProjectID)
	if err != nil {
		return err
	}
	recipients = withoutRecipient(recipients, event.AuthorUserID)
	return s.createForRecipients(ctx, recipients, todoports.CreateProjectTodoInput{
		OrganizationID: project.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindIssueCreated,
		TargetType:     "issue",
		TargetID:       formatID(event.IssueID),
		Title:          fmt.Sprintf("Issue #%d opened", event.IssueIID),
		Summary:        event.Title,
		ActionURL:      issueURL(project.OrganizationID, project.ID, event.IssueIID),
	})
}

func (s *Subscriber) handleProjectIssueAssigned(ctx context.Context, event issuedomain.ProjectIssueAssigned) error {
	if event.AssigneeUserID <= 0 || event.AssigneeUserID == event.ActorUserID {
		return nil
	}
	project, err := s.projectRepo.GetIncludingDeletedByID(ctx, event.ProjectID)
	if err != nil {
		return oops.In("todo").With("project_id", event.ProjectID, "issue_id", event.IssueID).Wrapf(err, "load project for issue assigned todo")
	}
	return s.createForRecipients(ctx, []int64{event.AssigneeUserID}, todoports.CreateProjectTodoInput{
		OrganizationID: project.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindIssueAssigned,
		TargetType:     "issue",
		TargetID:       formatID(event.IssueID),
		Title:          fmt.Sprintf("Assigned to issue #%d", event.IssueIID),
		Summary:        event.Title,
		ActionURL:      issueURL(project.OrganizationID, project.ID, event.IssueIID),
	})
}

func (s *Subscriber) handleProjectIssueCommented(ctx context.Context, event issuedomain.ProjectIssueCommented) error {
	if event.AuthorUserID <= 0 || event.AuthorUserID == event.CommentAuthorUserID {
		return nil
	}
	project, err := s.projectRepo.GetIncludingDeletedByID(ctx, event.ProjectID)
	if err != nil {
		return oops.In("todo").With("project_id", event.ProjectID, "issue_id", event.IssueID).Wrapf(err, "load project for issue comment todo")
	}
	return s.createForRecipients(ctx, []int64{event.AuthorUserID}, todoports.CreateProjectTodoInput{
		OrganizationID: project.OrganizationID,
		ProjectID:      event.ProjectID,
		Kind:           tododomain.ProjectTodoKindIssueCommented,
		TargetType:     "issue_comment",
		TargetID:       formatID(event.CommentID),
		Title:          fmt.Sprintf("New comment on issue #%d", event.IssueIID),
		Summary:        event.Title,
		ActionURL:      issueURL(project.OrganizationID, project.ID, event.IssueIID),
	})
}

func (s *Subscriber) projectRecipients(ctx context.Context, projectID int64) (projectdomain.Project, []int64, error) {
	project, err := s.projectRepo.GetIncludingDeletedByID(ctx, projectID)
	if err != nil {
		return projectdomain.Project{}, nil, oops.In("todo").With("project_id", projectID).Wrapf(err, "load project for todo recipients")
	}
	recipients := setx.NewOrderedSet[int64]()
	projectRecipients, err := s.projectMemberRecipients(ctx, projectID)
	if err != nil {
		return projectdomain.Project{}, nil, err
	}
	addRecipients(recipients, projectRecipients)
	organizationRecipients, err := s.organizationRecipients(ctx, project.OrganizationID)
	if err != nil {
		return projectdomain.Project{}, nil, err
	}
	addRecipients(recipients, organizationRecipients)
	return project, recipients.Values(), nil
}

func (s *Subscriber) projectMemberRecipients(ctx context.Context, projectID int64) ([]int64, error) {
	if s.projectMemberRepo == nil {
		return nil, nil
	}
	members, err := s.projectMemberRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("todo").With("project_id", projectID).Wrapf(err, "list project todo recipients")
	}
	userIDs := make([]int64, 0, members.Len())
	for _, member := range members.Values() {
		if member.UserID > 0 {
			userIDs = append(userIDs, member.UserID)
		}
	}
	return userIDs, nil
}

func (s *Subscriber) organizationRecipients(ctx context.Context, organizationID int64) ([]int64, error) {
	if s.organizationMemberRepo == nil {
		return nil, nil
	}
	members, err := s.organizationMemberRepo.ListByOrganizationID(ctx, organizationID)
	if err != nil {
		return nil, oops.In("todo").With("organization_id", organizationID).Wrapf(err, "list organization todo recipients")
	}
	return uniqueOrganizationMemberUserIDs(members.Values()), nil
}

func (s *Subscriber) createForRecipients(ctx context.Context, userIDs []int64, input todoports.CreateProjectTodoInput) error {
	for _, userID := range uniqueUserIDs(userIDs) {
		item := input
		item.UserID = userID
		if _, err := s.service.CreateIfNotExists(ctx, item); err != nil {
			return err
		}
	}
	return nil
}

func addRecipients(recipients *setx.OrderedSet[int64], userIDs []int64) {
	for _, userID := range userIDs {
		if userID > 0 {
			recipients.Add(userID)
		}
	}
}
func uniqueOrganizationMemberUserIDs(members []organizationdomain.OrganizationMember) []int64 {
	set := setx.NewOrderedSetWithCapacity[int64](len(members))
	for _, member := range members {
		if member.UserID > 0 {
			set.Add(member.UserID)
		}
	}
	return set.Values()
}

func uniqueUserIDs(userIDs []int64) []int64 {
	set := setx.NewOrderedSetWithCapacity[int64](len(userIDs))
	for _, userID := range userIDs {
		if userID > 0 {
			set.Add(userID)
		}
	}
	return set.Values()
}

func withoutRecipient(userIDs []int64, excluded int64) []int64 {
	items := make([]int64, 0, len(userIDs))
	for _, userID := range userIDs {
		if userID > 0 && userID != excluded {
			items = append(items, userID)
		}
	}
	return items
}
