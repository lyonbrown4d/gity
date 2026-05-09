package issue

import (
	"context"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	issueports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
)

type AssigneesInput struct {
	UserIDs []int64 `json:"user_ids"`
}

type AssigneesView struct {
	Issue     issuedomain.ProjectIssue           `json:"issue"`
	Assignees []issuedomain.ProjectIssueAssignee `json:"assignees"`
}

type LabelInput struct {
	Name  string `json:"name"`
	Color string `json:"color"`
}

type LabelsInput struct {
	Labels []LabelInput `json:"labels"`
}

type LabelsView struct {
	Issue  issuedomain.ProjectIssue        `json:"issue"`
	Labels []issuedomain.ProjectIssueLabel `json:"labels"`
}

func (s *Service) ListAssignees(ctx context.Context, projectID, issueIID int64) (AssigneesView, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return AssigneesView{}, err
	}
	assignees, err := s.listAssigneesByIssueID(ctx, issue.ID)
	if err != nil {
		return AssigneesView{}, err
	}
	return AssigneesView{Issue: issue, Assignees: assignees}, nil
}

func (s *Service) SetAssignees(ctx context.Context, projectID, issueIID int64, input AssigneesInput) (AssigneesView, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return AssigneesView{}, err
	}
	userIDs, err := s.normalizeAssigneeUserIDs(ctx, projectID, issueIID, input.UserIDs)
	if err != nil {
		return AssigneesView{}, err
	}
	if s.assigneeRepo == nil {
		return AssigneesView{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID).New("issue assignee repository is not configured")
	}
	items, err := s.assigneeRepo.ReplaceByIssueID(ctx, issue.ID, userIDs)
	if err != nil {
		return AssigneesView{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID).Wrapf(err, "replace issue assignees")
	}
	return AssigneesView{Issue: issue, Assignees: items.Values()}, nil
}

func (s *Service) ListLabels(ctx context.Context, projectID, issueIID int64) (LabelsView, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return LabelsView{}, err
	}
	labels, err := s.listLabelsByIssueID(ctx, issue.ID)
	if err != nil {
		return LabelsView{}, err
	}
	return LabelsView{Issue: issue, Labels: labels}, nil
}

func (s *Service) SetLabels(ctx context.Context, projectID, issueIID int64, input LabelsInput) (LabelsView, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return LabelsView{}, err
	}
	labels, err := normalizeLabelInputs(projectID, issueIID, input.Labels)
	if err != nil {
		return LabelsView{}, err
	}
	if s.labelRepo == nil {
		return LabelsView{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID).New("issue label repository is not configured")
	}
	items, err := s.labelRepo.ReplaceByIssueID(ctx, issue.ID, labels)
	if err != nil {
		return LabelsView{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID).Wrapf(err, "replace issue labels")
	}
	return LabelsView{Issue: issue, Labels: items.Values()}, nil
}

func (s *Service) listAssigneesByIssueID(ctx context.Context, issueID int64) ([]issuedomain.ProjectIssueAssignee, error) {
	if s.assigneeRepo == nil {
		return nil, oops.In("issue").With("issue_id", issueID).New("issue assignee repository is not configured")
	}
	assignees, err := s.assigneeRepo.ListByIssueID(ctx, issueID)
	if err != nil {
		return nil, oops.In("issue").With("issue_id", issueID).Wrapf(err, "list issue assignees")
	}
	return assignees.Values(), nil
}

func (s *Service) listLabelsByIssueID(ctx context.Context, issueID int64) ([]issuedomain.ProjectIssueLabel, error) {
	if s.labelRepo == nil {
		return nil, oops.In("issue").With("issue_id", issueID).New("issue label repository is not configured")
	}
	labels, err := s.labelRepo.ListByIssueID(ctx, issueID)
	if err != nil {
		return nil, oops.In("issue").With("issue_id", issueID).Wrapf(err, "list issue labels")
	}
	return labels.Values(), nil
}

func (s *Service) normalizeAssigneeUserIDs(ctx context.Context, projectID, issueIID int64, userIDs []int64) ([]int64, error) {
	seen := setx.NewOrderedSetWithCapacity[int64](len(userIDs))
	for _, userID := range userIDs {
		if userID <= 0 {
			err := oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "user_id", userID).New("assignee user id must be positive")
			return nil, apperror.BadRequest("assignee user id must be positive", err)
		}
		if seen.Contains(userID) {
			continue
		}
		if _, err := s.userRepo.GetByID(ctx, userID); err != nil {
			wrapped := oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "user_id", userID).Wrapf(err, "load issue assignee user")
			return nil, apperror.NotFound("issue assignee user not found", wrapped)
		}
		seen.Add(userID)
	}
	return seen.Values(), nil
}

func normalizeLabelInputs(projectID, issueIID int64, labels []LabelInput) ([]issueports.ProjectIssueLabelInput, error) {
	seen := setx.NewOrderedSetWithCapacity[string](len(labels))
	items := make([]issueports.ProjectIssueLabelInput, 0, len(labels))
	for _, label := range labels {
		name := strings.TrimSpace(label.Name)
		if name == "" {
			err := oops.In("issue").With("project_id", projectID, "issue_iid", issueIID).New("issue label name is required")
			return nil, apperror.BadRequest("issue label name is required", err)
		}
		if seen.Contains(name) {
			continue
		}
		seen.Add(name)
		items = append(items, issueports.ProjectIssueLabelInput{Name: name, Color: strings.TrimSpace(label.Color)})
	}
	return items, nil
}
