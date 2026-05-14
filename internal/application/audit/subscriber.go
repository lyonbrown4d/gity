package audit

import (
	"context"
	"encoding/json"
	"fmt"
	"strconv"

	"github.com/arcgolabs/eventx"
	auditports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

type Subscriber struct {
	repo        auditports.ProjectAuditEventRepository
	projectRepo auditports.ProjectRepository
	unsubscribe []func()
}

func NewSubscriber(repo auditports.ProjectAuditEventRepository, projectRepo auditports.ProjectRepository) *Subscriber {
	return &Subscriber{repo: repo, projectRepo: projectRepo}
}

func (s *Subscriber) Subscribe(bus eventx.BusRuntime) error {
	if bus == nil {
		return nil
	}
	return oops.Join(
		s.subscribeProjectCreated(bus),
		s.subscribeProjectDeleted(bus),
		s.subscribeProjectBranchProtectionChanged(bus),
		s.subscribeProjectBranchDeleted(bus),
		s.subscribeProjectMergeRequestMerged(bus),
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

func (s *Subscriber) subscribeProjectCreated(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectCreated)
	if err != nil {
		return oops.In("audit").Wrapf(err, "subscribe project created audit")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) subscribeProjectDeleted(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectDeleted)
	if err != nil {
		return oops.In("audit").Wrapf(err, "subscribe project deleted audit")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) subscribeProjectBranchProtectionChanged(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectBranchProtectionChanged)
	if err != nil {
		return oops.In("audit").Wrapf(err, "subscribe project branch protection audit")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) subscribeProjectBranchDeleted(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectBranchDeleted)
	if err != nil {
		return oops.In("audit").Wrapf(err, "subscribe project branch deleted audit")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) subscribeProjectMergeRequestMerged(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectMergeRequestMerged)
	if err != nil {
		return oops.In("audit").Wrapf(err, "subscribe project merge request merged audit")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) handleProjectCreated(ctx context.Context, event projectdomain.ProjectCreated) error {
	return s.create(ctx, auditports.CreateProjectAuditEventInput{
		ProjectID:      event.ProjectID,
		OrganizationID: event.OrganizationID,
		EventName:      event.Name(),
		Action:         "create_project",
		TargetType:     "project",
		TargetID:       formatID(event.ProjectID),
		Summary:        fmt.Sprintf("Project %s created", event.FullPath),
	}, event)
}

func (s *Subscriber) handleProjectDeleted(ctx context.Context, event projectdomain.ProjectDeleted) error {
	return s.create(ctx, auditports.CreateProjectAuditEventInput{
		ProjectID:      event.ProjectID,
		OrganizationID: event.OrganizationID,
		EventName:      event.Name(),
		Action:         "delete_project",
		TargetType:     "project",
		TargetID:       formatID(event.ProjectID),
		Summary:        fmt.Sprintf("Project %s marked pending delete", event.FullPath),
	}, event)
}

func (s *Subscriber) handleProjectBranchProtectionChanged(ctx context.Context, event projectdomain.ProjectBranchProtectionChanged) error {
	organizationID, err := s.organizationID(ctx, event.ProjectID)
	if err != nil {
		return err
	}
	action := "unprotect_branch"
	summary := "Branch protection removed from " + event.BranchName
	if event.Protected {
		action = "protect_branch"
		summary = "Branch protection updated for " + event.BranchName
	}
	return s.create(ctx, auditports.CreateProjectAuditEventInput{
		ProjectID:      event.ProjectID,
		OrganizationID: organizationID,
		EventName:      event.Name(),
		Action:         action,
		TargetType:     "branch_protection",
		TargetID:       event.BranchName,
		Summary:        summary,
	}, event)
}

func (s *Subscriber) handleProjectBranchDeleted(ctx context.Context, event projectdomain.ProjectBranchDeleted) error {
	organizationID, err := s.organizationID(ctx, event.ProjectID)
	if err != nil {
		return err
	}
	return s.create(ctx, auditports.CreateProjectAuditEventInput{
		ProjectID:      event.ProjectID,
		OrganizationID: organizationID,
		EventName:      event.Name(),
		Action:         "delete_branch",
		TargetType:     "branch",
		TargetID:       event.BranchName,
		Summary:        fmt.Sprintf("Branch %s deleted", event.BranchName),
	}, event)
}

func (s *Subscriber) handleProjectMergeRequestMerged(ctx context.Context, event mergedomain.ProjectMergeRequestMerged) error {
	organizationID, err := s.organizationID(ctx, event.ProjectID)
	if err != nil {
		return err
	}
	return s.create(ctx, auditports.CreateProjectAuditEventInput{
		ProjectID:      event.ProjectID,
		OrganizationID: organizationID,
		EventName:      event.Name(),
		Action:         "merge_merge_request",
		ActorUserID:    event.ActorUserID,
		TargetType:     "merge_request",
		TargetID:       formatID(event.MergeRequestID),
		Summary:        fmt.Sprintf("Merge request !%d merged into %s", event.MergeIID, event.TargetBranch),
	}, event)
}

func (s *Subscriber) organizationID(ctx context.Context, projectID int64) (int64, error) {
	project, err := s.projectRepo.GetIncludingDeletedByID(ctx, projectID)
	if err != nil {
		return 0, oops.In("audit").With("project_id", projectID).Wrapf(err, "load project for audit event")
	}
	return project.OrganizationID, nil
}

func (s *Subscriber) create(ctx context.Context, input auditports.CreateProjectAuditEventInput, event any) error {
	payload, err := json.Marshal(event)
	if err != nil {
		return oops.In("audit").With("project_id", input.ProjectID, "event_name", input.EventName).Wrapf(err, "marshal audit event payload")
	}
	input.Payload = string(payload)
	if _, err := s.repo.Create(ctx, input); err != nil {
		return oops.In("audit").With("project_id", input.ProjectID, "event_name", input.EventName, "action", input.Action).Wrapf(err, "create project audit event")
	}
	return nil
}

func formatID(id int64) string {
	return strconv.FormatInt(id, 10)
}
