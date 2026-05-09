package issue_test

import (
	"context"
	"encoding/base64"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace_member"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	projectissuerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue"
	projectissueattachmentrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue_attachment"
	projectissuecommentrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue_comment"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	infrastorage "github.com/DaiYuANg/gity/internal/infrastructure/storage"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestIssueFlow(t *testing.T) {
	t.Parallel()

	fixture := newIssueFixture(t)
	testutil.RequireNoError(t, pushFixtureCommit(fixture.ctx, fixture.repoRoot, fixture.projectFullPath+".git"), "push fixture commit")

	issueIID := createIssueRecord(t, fixture)
	assertIssueUpdate(t, fixture, issueIID)
	createIssueComment(t, fixture, issueIID)
	attachmentID := createIssueAttachment(t, fixture, issueIID)
	assertIssueCollections(t, fixture, issueIID)
	assertIssueAttachmentContent(t, fixture, issueIID, attachmentID)
}

type issueFixture struct {
	ctx             context.Context
	repoRoot        string
	projectID       int64
	projectFullPath string
	ownerID         int64
	service         *issueservice.Service
}

func newIssueFixture(t *testing.T) issueFixture {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-issue-test.db")
	db := testutil.Must(dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	))
	testutil.CleanupClose(t, "db", db)

	ctx := context.Background()
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	logger := slog.Default()
	namespaceRepository := testutil.Must(namespacerepo.NewRepository(db))
	namespaceMemberRepository := testutil.Must(namespacememberrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	projectBranchProtectionRepository := testutil.Must(projectbranchprotectionrepo.NewRepository(db))
	userRepository := testutil.Must(userrepo.NewRepository(db))
	userTokenRepository := testutil.Must(usertokenrepo.NewRepository(db))
	issueRepository := testutil.Must(projectissuerepo.NewRepository(db))
	commentRepository := testutil.Must(projectissuecommentrepo.NewRepository(db))
	attachmentRepository := testutil.Must(projectissueattachmentrepo.NewRepository(db))

	repoRoot := filepath.Join(t.TempDir(), "repos")
	storageRoot := filepath.Join(t.TempDir(), "storage")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})
	storage := testutil.Must(infrastorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: storageRoot}}))

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)
	issueSvc := issueservice.NewService(projectRepository, issueRepository, commentRepository, attachmentRepository, userRepository, storage)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	space := testutil.Must(namespaceSvc.Create(ctx, namespaceservice.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID}))
	project := testutil.Must(projectSvc.Create(ctx, projectservice.CreateInput{NamespaceID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"}))
	return issueFixture{ctx: ctx, repoRoot: repoRoot, projectID: project.ID, projectFullPath: project.FullPath, ownerID: owner.ID, service: issueSvc}
}

func createIssueRecord(t *testing.T, fixture issueFixture) int64 {
	t.Helper()
	issue := testutil.Must(fixture.service.CreateIssue(fixture.ctx, fixture.projectID, issueservice.CreateIssueInput{AuthorUserID: fixture.ownerID, Title: "first issue", Description: "seed issue"}))
	if issue.IID != 1 {
		t.Fatalf("expected first issue iid to be 1, got %d", issue.IID)
	}
	return issue.IID
}

func assertIssueUpdate(t *testing.T, fixture issueFixture, issueIID int64) {
	t.Helper()
	updated := testutil.Must(fixture.service.UpdateIssue(fixture.ctx, fixture.projectID, issueIID, issueservice.UpdateIssueInput{State: new("closed")}))
	if updated.State != "closed" {
		t.Fatalf("unexpected issue state: %s", updated.State)
	}
}

func createIssueComment(t *testing.T, fixture issueFixture, issueIID int64) {
	t.Helper()
	comment := testutil.Must(fixture.service.CreateComment(fixture.ctx, fixture.projectID, issueIID, issueservice.CreateCommentInput{AuthorUserID: fixture.ownerID, Body: "looks good"}))
	if comment.ID == 0 {
		t.Fatalf("expected comment id")
	}
}

func createIssueAttachment(t *testing.T, fixture issueFixture, issueIID int64) int64 {
	t.Helper()
	attachment := testutil.Must(fixture.service.CreateAttachment(fixture.ctx, fixture.projectID, issueIID, issueservice.CreateAttachmentInput{
		UploadedByUserID: fixture.ownerID,
		FileName:         "note.txt",
		ContentType:      "text/plain",
		ContentBase64:    base64.StdEncoding.EncodeToString([]byte("hello attachment")),
	}))
	if attachment.ByteSize != int64(len("hello attachment")) {
		t.Fatalf("unexpected attachment size: %d", attachment.ByteSize)
	}
	return attachment.ID
}

func assertIssueCollections(t *testing.T, fixture issueFixture, issueIID int64) {
	t.Helper()
	issues := testutil.Must(fixture.service.ListIssues(fixture.ctx, fixture.projectID))
	if len(issues) != 1 {
		t.Fatalf("expected one issue, got %d", len(issues))
	}
	comments := testutil.Must(fixture.service.ListComments(fixture.ctx, fixture.projectID, issueIID))
	if len(comments) != 1 || comments[0].Body != "looks good" {
		t.Fatalf("unexpected comments: %+v", comments)
	}
	attachments := testutil.Must(fixture.service.ListAttachments(fixture.ctx, fixture.projectID, issueIID))
	if len(attachments) != 1 || attachments[0].FileName != "note.txt" {
		t.Fatalf("unexpected attachments: %+v", attachments)
	}
}

func assertIssueAttachmentContent(t *testing.T, fixture issueFixture, issueIID, attachmentID int64) {
	t.Helper()
	attachmentContent := testutil.Must(fixture.service.GetAttachmentContent(fixture.ctx, fixture.projectID, issueIID, attachmentID))
	decoded := testutil.Must(base64.StdEncoding.DecodeString(attachmentContent.Content))
	if string(decoded) != "hello attachment" {
		t.Fatalf("unexpected attachment content: %s", string(decoded))
	}
}

func pushFixtureCommit(ctx context.Context, repoRoot, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree")
	if err := os.MkdirAll(worktree, 0o750); err != nil {
		return fmt.Errorf("create fixture worktree: %w", err)
	}
	if err := runGit(ctx, worktree, "init", "-b", "main"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "config", "user.name", "Gity Test"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "config", "user.email", "test@gity.dev"); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("# Hello Gity\n"), 0o600); err != nil {
		return fmt.Errorf("write fixture readme: %w", err)
	}
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Initial repository content"); err != nil {
		return err
	}

	absRepo := filepath.Join(repoRoot, filepath.FromSlash(repoPath))
	repoURL := "file:///" + filepath.ToSlash(absRepo)
	return runGit(ctx, worktree, "push", repoURL, "HEAD:refs/heads/main")
}

func runGit(ctx context.Context, dir string, args ...string) error {
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = dir
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git %v: %w: %s", args, err, strings.TrimSpace(string(output)))
	}
	return nil
}
