package mergerequest

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/DaiYuANg/gity/internal/config"
	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/repository/projectbranchprotection"
	projectjobrepo "github.com/DaiYuANg/gity/internal/repository/projectjob"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/repository/projectmergerequest"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/repository/projectpipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/repository/projectpipelinejob"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
	jobservice "github.com/DaiYuANg/gity/internal/service/job"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	pipelineservice "github.com/DaiYuANg/gity/internal/service/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestMergeRequestFlow(t *testing.T) {
	t.Parallel()

	dbPath := filepath.Join(t.TempDir(), "gity-mr-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	logger := slog.Default()
	namespaceRepository, _ := namespacerepo.NewRepository(db)
	namespaceMemberRepository, _ := namespacememberrepo.NewRepository(db)
	projectRepository, _ := projectrepo.NewRepository(db)
	projectBranchProtectionRepository, _ := projectbranchprotectionrepo.NewRepository(db)
	mergeRequestRepository, _ := projectmergerequestrepo.NewRepository(db)
	pipelineRepository, _ := projectpipelinerepo.NewRepository(db)
	userRepository, _ := userrepo.NewRepository(db)
	userTokenRepository, _ := usertokenrepo.NewRepository(db)

	repoRoot := filepath.Join(t.TempDir(), "repos")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)
	mergeRequestSvc := NewService(projectRepository, mergeRequestRepository, userRepository, gitRepository, runner, NewPipelineDeps(pipelineRepository, nil))

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
	if err := pushFixtureBranches(ctx, repoRoot, project.FullPath+".git"); err != nil {
		t.Fatalf("push fixture branches: %v", err)
	}

	mr, err := mergeRequestSvc.Create(ctx, project.ID, CreateInput{AuthorUserID: owner.ID, Title: "merge feature", Description: "merge feature into main", SourceBranch: "feature", TargetBranch: "main"})
	if err != nil {
		t.Fatalf("create merge request: %v", err)
	}
	if mr.IID != 1 {
		t.Fatalf("expected first merge request iid to be 1, got %d", mr.IID)
	}
	diff, err := mergeRequestSvc.GetDiff(ctx, project.ID, mr.IID)
	if err != nil {
		t.Fatalf("get merge request diff: %v", err)
	}
	if !strings.Contains(diff.Diff, "feature.txt") {
		t.Fatalf("expected diff to include feature file, got %s", diff.Diff)
	}
	merged, err := mergeRequestSvc.Merge(ctx, project.ID, mr.IID, MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev"})
	if err != nil {
		t.Fatalf("merge request: %v", err)
	}
	if merged.State != "merged" {
		t.Fatalf("expected merged state, got %s", merged.State)
	}

	closedMR, err := mergeRequestSvc.Create(ctx, project.ID, CreateInput{AuthorUserID: owner.ID, Title: "close feature", Description: "close feature", SourceBranch: "feature", TargetBranch: "main"})
	if err != nil {
		t.Fatalf("create second merge request: %v", err)
	}
	updated, err := mergeRequestSvc.Update(ctx, project.ID, closedMR.IID, UpdateInput{State: stringPtr("closed")})
	if err != nil {
		t.Fatalf("update merge request: %v", err)
	}
	if updated.State != "closed" {
		t.Fatalf("unexpected merge request state: %s", updated.State)
	}

	items, err := mergeRequestSvc.List(ctx, project.ID)
	if err != nil {
		t.Fatalf("list merge requests: %v", err)
	}
	if len(items) != 2 || items[0].SourceBranch != "feature" || items[0].TargetBranch != "main" {
		t.Fatalf("unexpected merge requests: %+v", items)
	}
}

func TestMergeRequestMergeRequiresSuccessfulPipelineWhenCIConfigExists(t *testing.T) {
	t.Parallel()

	dbPath := filepath.Join(t.TempDir(), "gity-mr-checks-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	logger := slog.Default()
	namespaceRepository, _ := namespacerepo.NewRepository(db)
	namespaceMemberRepository, _ := namespacememberrepo.NewRepository(db)
	projectRepository, _ := projectrepo.NewRepository(db)
	projectBranchProtectionRepository, _ := projectbranchprotectionrepo.NewRepository(db)
	mergeRequestRepository, _ := projectmergerequestrepo.NewRepository(db)
	pipelineRepository, _ := projectpipelinerepo.NewRepository(db)
	pipelineJobRepository, _ := projectpipelinejobrepo.NewRepository(db)
	jobRepository, _ := projectjobrepo.NewRepository(db)
	userRepository, _ := userrepo.NewRepository(db)
	userTokenRepository, _ := usertokenrepo.NewRepository(db)

	repoRoot := filepath.Join(t.TempDir(), "repos")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)
	jobSvc := jobservice.NewService(logger, projectRepository, jobRepository, nil, nil, nil)
	pipelineSvc := pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, gitRepository)
	mergeRequestSvc := NewService(projectRepository, mergeRequestRepository, userRepository, gitRepository, runner, NewPipelineDeps(pipelineRepository, pipelineSvc))

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
	if err := pushFixtureBranches(ctx, repoRoot, project.FullPath+".git"); err != nil {
		t.Fatalf("push fixture branches: %v", err)
	}
	if err := runner.CreateFileCommit(ctx, project.FullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "feature",
		FilePath:    ".gity-ci.plano",
		Content:     mergeRequestCIConfig(),
		Message:     "Add CI config",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}); err != nil {
		t.Fatalf("add ci config: %v", err)
	}
	sourceBranch := findBranch(t, ctx, gitRepository, project.FullPath+".git", project.DefaultBranch, "feature")

	mr, err := mergeRequestSvc.Create(ctx, project.ID, CreateInput{AuthorUserID: owner.ID, Title: "merge feature with checks", SourceBranch: "feature", TargetBranch: "main"})
	if err != nil {
		t.Fatalf("create merge request: %v", err)
	}
	checks, err := mergeRequestSvc.GetChecks(ctx, project.ID, mr.IID)
	if err != nil {
		t.Fatalf("get missing checks: %v", err)
	}
	if !checks.Required || checks.Mergeable || checks.Status != "missing" {
		t.Fatalf("expected missing required checks: %+v", checks)
	}
	if _, err := mergeRequestSvc.Merge(ctx, project.ID, mr.IID, MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev"}); err == nil {
		t.Fatalf("expected merge to be blocked before pipeline exists")
	}

	pipeline, err := pipelineRepository.Create(ctx, projectpipelinerepo.CreateInput{
		ProjectID:     project.ID,
		Name:          "merge-request",
		Source:        "push",
		RefName:       "feature",
		CommitSHA:     sourceBranch.Hash,
		Status:        projectpipelinerepo.StatusFailed,
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: mergeRequestCIConfig(),
	})
	if err != nil {
		t.Fatalf("create failed pipeline: %v", err)
	}
	checks, err = mergeRequestSvc.GetChecks(ctx, project.ID, mr.IID)
	if err != nil {
		t.Fatalf("get failed checks: %v", err)
	}
	if checks.Mergeable || checks.Status != projectpipelinerepo.StatusFailed || checks.Pipeline == nil {
		t.Fatalf("expected failed checks: %+v", checks)
	}
	if err := pipelineRepository.UpdateStatus(ctx, pipeline, projectpipelinerepo.StatusSucceeded); err != nil {
		t.Fatalf("mark pipeline succeeded: %v", err)
	}
	merged, err := mergeRequestSvc.Merge(ctx, project.ID, mr.IID, MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev"})
	if err != nil {
		t.Fatalf("merge after successful pipeline: %v", err)
	}
	if merged.State != "merged" {
		t.Fatalf("expected merged state, got %s", merged.State)
	}
	targetBranch := findBranch(t, ctx, gitRepository, project.FullPath+".git", project.DefaultBranch, "main")
	targetPipeline, err := pipelineRepository.GetLatestByProjectRefCommit(ctx, project.ID, "main", targetBranch.Hash)
	if err != nil {
		t.Fatalf("get target branch pipeline: %v", err)
	}
	if targetPipeline.Source != "push" || targetPipeline.CommitSHA != targetBranch.Hash {
		t.Fatalf("unexpected target branch pipeline: %+v", targetPipeline)
	}
}

func stringPtr(value string) *string {
	return &value
}

func mergeRequestCIConfig() string {
	return `
pipeline {
  name = "merge-request"
}

stage test {
  run {
    shell("echo ok")
  }
}
`
}

func findBranch(t *testing.T, ctx context.Context, gitRepository *gitrepo.Service, repoPath string, defaultBranch string, branchName string) gitrepo.Branch {
	t.Helper()
	branches, err := gitRepository.ListBranches(ctx, repoPath, defaultBranch)
	if err != nil {
		t.Fatalf("list branches: %v", err)
	}
	for _, branch := range branches {
		if branch.Name == branchName {
			return branch
		}
	}
	t.Fatalf("branch %s not found: %+v", branchName, branches)
	return gitrepo.Branch{}
}

func pushFixtureBranches(ctx context.Context, repoRoot string, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree-mr")
	if err := os.MkdirAll(worktree, 0o755); err != nil {
		return err
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
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("# Hello Gity\n"), 0o644); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Initial repository content"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "branch", "feature"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "checkout", "feature"); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(worktree, "feature.txt"), []byte("feature branch\n"), 0o644); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Add feature content"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "checkout", "main"); err != nil {
		return err
	}

	absRepo := filepath.Join(repoRoot, filepath.FromSlash(repoPath))
	repoURL := "file:///" + filepath.ToSlash(absRepo)
	if err := runGit(ctx, worktree, "push", repoURL, "main:refs/heads/main"); err != nil {
		return err
	}
	return runGit(ctx, worktree, "push", repoURL, "feature:refs/heads/feature")
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
