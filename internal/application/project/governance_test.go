package project_test

import (
	"testing"

	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	"github.com/DaiYuANg/gity/internal/testutil"
)

func branchProtected(branches []projectservice.Branch, branchName string) bool {
	for _, branch := range branches {
		if branch.Name == branchName {
			return branch.IsProtected
		}
	}
	return false
}

func assertProjectDeletionGovernance(t *testing.T, fixture *projectFixture) {
	t.Helper()

	if err := fixture.projectService.Delete(fixture.ctx, fixture.projectID, projectservice.DeleteInput{Confirmation: fixture.projectFullPath + "-wrong"}); err == nil {
		t.Fatalf("expected delete confirmation mismatch to fail")
	}
	projects := testutil.Must(fixture.projectService.List(fixture.ctx, &fixture.organizationID))
	if projects.Len() != 1 {
		t.Fatalf("expected project to remain visible after failed delete, got %d", projects.Len())
	}
	testutil.RequireNoError(t, fixture.projectService.Delete(fixture.ctx, fixture.projectID, projectservice.DeleteInput{Confirmation: fixture.projectFullPath}), "delete project")
	projects = testutil.Must(fixture.projectService.List(fixture.ctx, &fixture.organizationID))
	if projects.Len() != 0 {
		t.Fatalf("expected pending delete project to be hidden, got %d", projects.Len())
	}
	if _, err := fixture.projectService.GetByID(fixture.ctx, fixture.projectID); err == nil {
		t.Fatalf("expected pending delete project to be hidden from get")
	}
}
