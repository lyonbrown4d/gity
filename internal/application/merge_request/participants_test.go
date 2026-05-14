package mergerequest_test

import (
	"testing"

	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	"github.com/lyonbrown4d/gity/internal/testutil"
)

func TestMergeRequestParticipantsFlow(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, false)
	mrIID := assertCreateMergeRequest(t, fixture, "assign review")
	assertMergeRequestParticipants(t, fixture, mrIID)
}

func assertMergeRequestParticipants(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	reviewers := testutil.Must(fixture.mergeRequestService.SetReviewers(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.ParticipantsInput{UserIDs: []int64{fixture.reviewerID, fixture.ownerID, fixture.reviewerID}}))
	if len(reviewers.Participants) != 2 {
		t.Fatalf("expected duplicate reviewer ids to be de-duplicated, got %+v", reviewers.Participants)
	}
	assignees := testutil.Must(fixture.mergeRequestService.SetAssignees(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.ParticipantsInput{UserIDs: []int64{fixture.ownerID}}))
	if len(assignees.Participants) != 3 {
		t.Fatalf("expected reviewers and assignees to be listed together, got %+v", assignees.Participants)
	}
	listed := testutil.Must(fixture.mergeRequestService.ListParticipants(fixture.ctx, fixture.projectID, mrIID))
	if len(listed.Participants) != 3 || listed.MergeRequest.IID != mrIID {
		t.Fatalf("unexpected merge request participants: %+v", listed)
	}
	if _, err := fixture.mergeRequestService.SetReviewers(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.ParticipantsInput{UserIDs: []int64{-1}}); err == nil {
		t.Fatalf("expected invalid participant user id to fail")
	}
}
