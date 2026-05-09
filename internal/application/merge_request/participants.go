package mergerequest

import (
	"context"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
)

type ParticipantsInput struct {
	UserIDs []int64 `json:"user_ids"`
}

type ParticipantsView struct {
	MergeRequest mergedomain.ProjectMergeRequest              `json:"merge_request"`
	Participants []mergedomain.ProjectMergeRequestParticipant `json:"participants"`
}

func (s *Service) ListParticipants(ctx context.Context, projectID, mergeIID int64) (ParticipantsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return ParticipantsView{}, err
	}
	participants, err := s.listParticipantsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return ParticipantsView{}, err
	}
	return ParticipantsView{MergeRequest: mr, Participants: participants}, nil
}

func (s *Service) SetReviewers(ctx context.Context, projectID, mergeIID int64, input ParticipantsInput) (ParticipantsView, error) {
	return s.replaceParticipants(ctx, projectID, mergeIID, mergedomain.ProjectMergeRequestParticipantRoleReviewer, input.UserIDs)
}

func (s *Service) SetAssignees(ctx context.Context, projectID, mergeIID int64, input ParticipantsInput) (ParticipantsView, error) {
	return s.replaceParticipants(ctx, projectID, mergeIID, mergedomain.ProjectMergeRequestParticipantRoleAssignee, input.UserIDs)
}

func (s *Service) replaceParticipants(ctx context.Context, projectID, mergeIID int64, role string, userIDs []int64) (ParticipantsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return ParticipantsView{}, err
	}
	normalizedUserIDs, err := s.normalizeParticipantUserIDs(ctx, projectID, mergeIID, role, userIDs)
	if err != nil {
		return ParticipantsView{}, err
	}
	if s.participantRepo == nil {
		return ParticipantsView{}, oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "role", role).New("merge request participant repository is not configured")
	}
	if _, replaceErr := s.participantRepo.ReplaceByMergeRequestAndRole(ctx, mr.ID, role, normalizedUserIDs); replaceErr != nil {
		return ParticipantsView{}, oops.In("merge_request").With("project_id", projectID, "merge_request_id", mr.ID, "merge_iid", mergeIID, "role", role).Wrapf(replaceErr, "replace merge request participants")
	}
	participants, err := s.listParticipantsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return ParticipantsView{}, err
	}
	return ParticipantsView{MergeRequest: mr, Participants: participants}, nil
}

func (s *Service) listParticipantsByMergeRequestID(ctx context.Context, mergeRequestID int64) ([]mergedomain.ProjectMergeRequestParticipant, error) {
	if s.participantRepo == nil {
		return nil, oops.In("merge_request").With("merge_request_id", mergeRequestID).New("merge request participant repository is not configured")
	}
	participants, err := s.participantRepo.ListByMergeRequestID(ctx, mergeRequestID)
	if err != nil {
		return nil, oops.In("merge_request").With("merge_request_id", mergeRequestID).Wrapf(err, "list merge request participants")
	}
	return participants.Values(), nil
}

func (s *Service) normalizeParticipantUserIDs(ctx context.Context, projectID, mergeIID int64, role string, userIDs []int64) ([]int64, error) {
	seen := setx.NewOrderedSetWithCapacity[int64](len(userIDs))
	for _, userID := range userIDs {
		if userID <= 0 {
			err := oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "role", role, "user_id", userID).New("participant user id must be positive")
			return nil, apperror.BadRequest("participant user id must be positive", err)
		}
		if seen.Contains(userID) {
			continue
		}
		if _, err := s.userRepo.GetByID(ctx, userID); err != nil {
			wrapped := oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "role", role, "user_id", userID).Wrapf(err, "load participant user")
			return nil, apperror.NotFound("merge request participant user not found", wrapped)
		}
		seen.Add(userID)
	}
	return seen.Values(), nil
}
