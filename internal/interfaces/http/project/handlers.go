package project

import (
	"context"
	"strconv"
	"strings"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_repo"
)

func (e *Endpoint) listProjects(ctx context.Context, in *projectsInput) (*projectOutput, error) {
	principal, err := e.requirePermissionPrincipal(ctx, in.Authorization)
	if err != nil {
		return nil, err
	}
	organizationFilter := projectOrganizationFilter(in)
	items, err := e.service.List(ctx, organizationFilter)
	if err != nil {
		return nil, err
	}
	idFilter := parseIDFilter(in.IDs)
	projectItems := items.Values()
	visible := make([]projectdomain.Project, 0, len(projectItems))
	for i := range projectItems {
		item := projectItems[i]
		if idFilter.Len() > 0 && !idFilter.Contains(item.ID) {
			continue
		}
		allowed, err := e.authRuntime.CanProjectAction(ctx, principal, projectAuthScope(item), infraauth.ProjectActionRead)
		if err != nil {
			return nil, err
		}
		if allowed {
			visible = append(visible, item)
		}
	}
	views := collectionlist.MapList(collectionlist.NewList(visible...), func(_ int, item projectdomain.Project) repositoryView {
		return toRepositoryView(item, e.settings)
	}).Values()
	return &projectOutput{Body: views}, nil
}

func (e *Endpoint) getProject(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toRepositoryView(item, e.settings)}, nil
}

func (e *Endpoint) createProject(ctx context.Context, in *createProjectInput) (*projectOutput, error) {
	input := buildCreateProjectInput(in)
	if err := e.requireProjectCreate(ctx, in.Authorization, input.OrganizationID); err != nil {
		return nil, err
	}
	item, err := e.service.Create(ctx, input)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toRepositoryView(item, e.settings)}, nil
}

func (e *Endpoint) deleteProject(ctx context.Context, in *deleteProjectInput) (*projectOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	if err := e.service.Delete(ctx, in.ID, projectservice.DeleteInput{Confirmation: in.Body.Confirmation}); err != nil {
		return nil, err
	}
	item.Status = projectdomain.ProjectStatusPendingDelete
	item.DeletedAt = time.Now().UTC()
	return &projectOutput{Body: toRepositoryView(item, e.settings)}, nil
}

func (e *Endpoint) listMembers(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
	items, err := e.memberService.ListMembers(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: items}, nil
}

func (e *Endpoint) createMember(ctx context.Context, in *createProjectMemberInput) (*projectOutput, error) {
	item, err := e.memberService.UpsertMember(ctx, in.ID, projectservice.MemberInput{UserID: in.Body.UserID, Role: in.Body.Role})
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: item}, nil
}

func (e *Endpoint) upsertMember(ctx context.Context, in *upsertProjectMemberInput) (*projectOutput, error) {
	userID := in.UserID
	if userID <= 0 {
		userID = in.Body.UserID
	}
	item, err := e.memberService.UpsertMember(ctx, in.ID, projectservice.MemberInput{UserID: userID, Role: in.Body.Role})
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: item}, nil
}

func (e *Endpoint) deleteMember(ctx context.Context, in *projectMemberInput) (*projectOutput, error) {
	if err := e.memberService.DeleteMember(ctx, in.ID, in.UserID); err != nil {
		return nil, err
	}
	return &projectOutput{Body: map[string]any{"status": "deleted"}}, nil
}

func (e *Endpoint) listBranches(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
	items, err := e.service.ListBranches(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item projectservice.Branch) repositoryBranchView {
		return toRepositoryBranchView(in.ID, item)
	}).Values()
	return &projectOutput{Body: views}, nil
}

func (e *Endpoint) createBranch(ctx context.Context, in *createBranchInput) (*projectOutput, error) {
	if err := e.requireDirectBranchPush(ctx, in.Authorization, in.ID, in.Body.Name); err != nil {
		return nil, err
	}
	item, err := e.service.CreateBranch(ctx, in.ID, in.Body.Name, in.Body.SourceRef)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toRepositoryBranchView(in.ID, item)}, nil
}

func (e *Endpoint) deleteBranch(ctx context.Context, in *branchProtectionInput) (*projectOutput, error) {
	if err := e.service.DeleteBranch(ctx, in.ID, in.BranchName); err != nil {
		return nil, err
	}
	return &projectOutput{Body: map[string]any{"status": "deleted"}}, nil
}

func (e *Endpoint) protectBranch(ctx context.Context, in *branchProtectionInput) (*projectOutput, error) {
	return e.setBranchProtection(ctx, in, true)
}

func (e *Endpoint) unprotectBranch(ctx context.Context, in *branchProtectionInput) (*projectOutput, error) {
	return e.setBranchProtection(ctx, in, false)
}

func (e *Endpoint) listBranchProtections(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
	items, err := e.service.ListBranchProtections(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item projectservice.BranchProtection) branchProtectionView {
		return *toBranchProtectionView(&item)
	}).Values()
	return &projectOutput{Body: views}, nil
}

func (e *Endpoint) upsertBranchProtection(ctx context.Context, in *upsertBranchProtectionInput) (*projectOutput, error) {
	item, err := e.service.UpsertBranchProtection(ctx, in.ID, projectservice.BranchProtectionInput{
		BranchName:             in.BranchName,
		RuleType:               in.Body.RuleType,
		PushAccessLevel:        in.Body.PushAccessLevel,
		MergeAccessLevel:       in.Body.MergeAccessLevel,
		RequireMergeRequest:    in.Body.RequireMergeRequest,
		RequirePipelineSuccess: in.Body.RequirePipelineSuccess,
		AllowForcePush:         in.Body.AllowForcePush,
		AllowDelete:            in.Body.AllowDelete,
	})
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toBranchProtectionView(&item)}, nil
}

func (e *Endpoint) listCommits(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
	refName := repositoryRefName(in.Ref, in.Branch)
	items, err := e.service.ListCommits(ctx, in.ID, refName, in.Limit)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item gitrepo.Commit) repositoryCommitView {
		return toRepositoryCommitView(in.ID, refName, item)
	}).Values()
	return &projectOutput{Body: views}, nil
}

func (e *Endpoint) listTree(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
	items, err := e.service.ListTree(ctx, in.ID, repositoryRefName(in.Ref, in.Branch), in.Path)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item gitrepo.TreeEntry) repositoryTreeEntryView {
		return toRepositoryTreeEntryView(item)
	}).Values()
	return &projectOutput{Body: views}, nil
}

func (e *Endpoint) getBlob(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
	item, err := e.service.GetBlob(ctx, in.ID, repositoryRefName(in.Ref, in.Branch), in.Path)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toRepositoryBlobView(item)}, nil
}

func (e *Endpoint) getReadme(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
	item, err := e.service.GetReadme(ctx, in.ID, repositoryRefName(in.Ref, in.Branch))
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toRepositoryBlobView(item)}, nil
}

func (e *Endpoint) searchRepository(ctx context.Context, in *projectRepositorySearchInput) (*projectOutput, error) {
	items, err := e.service.Search(ctx, in.ID, repositoryRefName(in.Ref, in.Branch), in.Query, in.Path, in.Limit, in.MaxFiles, in.MaxFileSize, in.MatchCase, in.Regex)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: items}, nil
}

func (e *Endpoint) createFileCommit(ctx context.Context, in *createFileCommitInput) (*projectOutput, error) {
	branchName := strings.TrimSpace(in.Body.BranchName)
	if err := e.requireDirectBranchPush(ctx, in.Authorization, in.ID, branchName); err != nil {
		return nil, err
	}
	if err := e.service.CreateFileCommit(ctx, in.ID, buildCreateFileCommitInput(in, branchName)); err != nil {
		return nil, err
	}
	body := map[string]any{"status": "created"}
	e.attachPipelineTrigger(ctx, body, in.ID, branchName)
	return &projectOutput{Body: body}, nil
}

func (e *Endpoint) languages(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
	branchName := repositoryRefName(in.Branch, in.Ref)
	analysis, err := e.service.AnalyzeLanguages(ctx, in.ID, branchName)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: repositoryLanguagesView{
		RepositoryID: strconv.FormatInt(in.ID, 10),
		BranchName:   branchName,
		Status:       "analyzed",
		Revision:     analysis.Revision,
		AnalyzedAt:   time.Now().UTC().Format("2006-01-02T15:04:05Z"),
		TotalBytes:   analysis.TotalBytes,
		Languages:    analysis.Languages,
	}}, nil
}

func (e *Endpoint) setBranchProtection(ctx context.Context, in *branchProtectionInput, protected bool) (*projectOutput, error) {
	item, err := e.service.SetBranchProtection(ctx, in.ID, in.BranchName, protected)
	if err != nil {
		return nil, err
	}
	return &projectOutput{Body: toRepositoryBranchView(in.ID, item)}, nil
}
