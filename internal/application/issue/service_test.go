package issue

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

	dbPath := filepath.Join(t.TempDir(), "gity-issue-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	testutil.CleanupClose(t, "db", db)

	ctx := context.Background()
	if schemaErr := core.EnsureSchema(ctx, db); schemaErr != nil {
		t.Fatalf("ensure schema: %v", schemaErr)
	}

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
	storage, err := infrastorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: storageRoot}})
	if err != nil {
		t.Fatalf("new storage service: %v", err)
	}

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)
	issueSvc := NewService(projectRepository, issueRepository, commentRepository, attachmentRepository, userRepository, storage)

	owner, err := userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"})
	if err != nil {
		t.Fatalf("create user: %v", err)
	}
	space, err := namespaceSvc.Create(ctx, namespaceservice.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectSvc.Create(ctx, projectservice.CreateInput{NamespaceID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"})
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	if pushErr := pushFixtureCommit(ctx, repoRoot, project.FullPath+".git"); pushErr != nil {
		t.Fatalf("push fixture commit: %v", pushErr)
	}

	issue, err := issueSvc.CreateIssue(ctx, project.ID, CreateIssueInput{AuthorUserID: owner.ID, Title: "first issue", Description: "seed issue"})
	if err != nil {
		t.Fatalf("create issue: %v", err)
	}
	if issue.IID != 1 {
		t.Fatalf("expected first issue iid to be 1, got %d", issue.IID)
	}

	updated, err := issueSvc.UpdateIssue(ctx, project.ID, issue.IID, UpdateIssueInput{State: new("closed")})
	if err != nil {
		t.Fatalf("update issue: %v", err)
	}
	if updated.State != "closed" {
		t.Fatalf("unexpected issue state: %s", updated.State)
	}

	comment, err := issueSvc.CreateComment(ctx, project.ID, issue.IID, CreateCommentInput{AuthorUserID: owner.ID, Body: "looks good"})
	if err != nil {
		t.Fatalf("create comment: %v", err)
	}
	if comment.ID == 0 {
		t.Fatalf("expected comment id")
	}

	attachment, err := issueSvc.CreateAttachment(ctx, project.ID, issue.IID, CreateAttachmentInput{
		UploadedByUserID: owner.ID,
		FileName:         "note.txt",
		ContentType:      "text/plain",
		ContentBase64:    base64.StdEncoding.EncodeToString([]byte("hello attachment")),
	})
	if err != nil {
		t.Fatalf("create attachment: %v", err)
	}
	if attachment.ByteSize != int64(len("hello attachment")) {
		t.Fatalf("unexpected attachment size: %d", attachment.ByteSize)
	}

	issues, err := issueSvc.ListIssues(ctx, project.ID)
	if err != nil {
		t.Fatalf("list issues: %v", err)
	}
	if len(issues) != 1 {
		t.Fatalf("expected one issue, got %d", len(issues))
	}

	comments, err := issueSvc.ListComments(ctx, project.ID, issue.IID)
	if err != nil {
		t.Fatalf("list comments: %v", err)
	}
	if len(comments) != 1 || comments[0].Body != "looks good" {
		t.Fatalf("unexpected comments: %+v", comments)
	}

	attachments, err := issueSvc.ListAttachments(ctx, project.ID, issue.IID)
	if err != nil {
		t.Fatalf("list attachments: %v", err)
	}
	if len(attachments) != 1 || attachments[0].FileName != "note.txt" {
		t.Fatalf("unexpected attachments: %+v", attachments)
	}

	attachmentContent, err := issueSvc.GetAttachmentContent(ctx, project.ID, issue.IID, attachment.ID)
	if err != nil {
		t.Fatalf("get attachment content: %v", err)
	}
	decoded, err := base64.StdEncoding.DecodeString(attachmentContent.Content)
	if err != nil {
		t.Fatalf("decode attachment content: %v", err)
	}
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
