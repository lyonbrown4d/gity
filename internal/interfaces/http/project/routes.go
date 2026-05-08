package project

import (
	"context"

	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"strconv"
	"strings"
	"time"

	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	"github.com/DaiYuANg/gity/internal/config"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

type Endpoint struct {
	service         *projectservice.Service
	settings        config.Settings
	authRuntime     *infraauth.Runtime
	pipelineService *pipelineservice.Service
}

func NewEndpoint(service *projectservice.Service, settings config.Settings, authRuntime *infraauth.Runtime, pipelineService *pipelineservice.Service) *Endpoint {
	return &Endpoint{service: service, settings: settings, authRuntime: authRuntime, pipelineService: pipelineService}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Projects", "Projects", "Project and repository APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	settings := e.settings
	projectWrite := e.projectScope

	listProjects := func(ctx context.Context, in *projectsInput) (*projectOutput, error) {
		namespaceID := in.NamespaceID
		if namespaceID == 0 {
			namespaceID = in.OrganizationID
		}
		var namespaceFilter *int64
		if namespaceID > 0 {
			namespaceFilter = &namespaceID
		}
		items, err := service.List(ctx, namespaceFilter)
		if err != nil {
			return nil, err
		}
		idFilter := parseIDFilter(in.IDs)
		views := collectionlist.FilterMapList(items, func(_ int, item projectdomain.Project) (repositoryView, bool) {
			if idFilter.Len() > 0 && !idFilter.Contains(item.ID) {
				return repositoryView{}, false
			}
			return toRepositoryView(item, settings), true
		}).Values()
		return &projectOutput{Body: views}, nil
	}

	getProject := func(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryView(item, settings)}, nil
	}

	createProject := func(ctx context.Context, in *createProjectInput) (*projectOutput, error) {
		namespaceID := in.Body.NamespaceID
		if namespaceID == 0 {
			namespaceID = in.Body.OrganizationID
		}
		item, err := service.Create(ctx, projectservice.CreateInput{
			NamespaceID:   namespaceID,
			Name:          in.Body.Name,
			PathKey:       firstNonEmpty(in.Body.PathKey, in.Body.Key),
			Visibility:    in.Body.Visibility,
			Description:   in.Body.Description,
			DefaultBranch: in.Body.DefaultBranch,
		})
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryView(item, settings)}, nil
	}

	deleteProject := func(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		if err := service.Delete(ctx, in.ID); err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryView(item, settings)}, nil
	}

	listBranches := func(ctx context.Context, in *projectByIDInput) (*projectOutput, error) {
		items, err := service.ListBranches(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item projectservice.Branch) repositoryBranchView {
			return toRepositoryBranchView(in.ID, item)
		}).Values()
		return &projectOutput{Body: views}, nil
	}

	createBranch := func(ctx context.Context, in *createBranchInput) (*projectOutput, error) {
		item, err := service.CreateBranch(ctx, in.ID, in.Body.Name, in.Body.SourceRef)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryBranchView(in.ID, item)}, nil
	}

	protectBranch := func(ctx context.Context, in *branchProtectionInput) (*projectOutput, error) {
		item, err := service.SetBranchProtection(ctx, in.ID, in.BranchName, true)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryBranchView(in.ID, item)}, nil
	}
	unprotectBranch := func(ctx context.Context, in *branchProtectionInput) (*projectOutput, error) {
		item, err := service.SetBranchProtection(ctx, in.ID, in.BranchName, false)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryBranchView(in.ID, item)}, nil
	}

	listCommits := func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		refName := strings.TrimSpace(in.Ref)
		if refName == "" {
			refName = strings.TrimSpace(in.Branch)
		}
		items, err := service.ListCommits(ctx, in.ID, refName, in.Limit)
		if err != nil {
			return nil, err
		}
		views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item gitrepo.Commit) repositoryCommitView {
			return toRepositoryCommitView(in.ID, refName, item)
		}).Values()
		return &projectOutput{Body: views}, nil
	}

	listTree := func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		refName := strings.TrimSpace(in.Ref)
		if refName == "" {
			refName = strings.TrimSpace(in.Branch)
		}
		items, err := service.ListTree(ctx, in.ID, refName, in.Path)
		if err != nil {
			return nil, err
		}
		views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item gitrepo.TreeEntry) repositoryTreeEntryView {
			return toRepositoryTreeEntryView(item)
		}).Values()
		return &projectOutput{Body: views}, nil
	}

	getBlob := func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		refName := strings.TrimSpace(in.Ref)
		if refName == "" {
			refName = strings.TrimSpace(in.Branch)
		}
		item, err := service.GetBlob(ctx, in.ID, refName, in.Path)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryBlobView(item)}, nil
	}

	getReadme := func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		refName := strings.TrimSpace(in.Ref)
		if refName == "" {
			refName = strings.TrimSpace(in.Branch)
		}
		item, err := service.GetReadme(ctx, in.ID, refName)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: toRepositoryBlobView(item)}, nil
	}

	searchRepository := func(ctx context.Context, in *projectRepositorySearchInput) (*projectOutput, error) {
		refName := strings.TrimSpace(in.Ref)
		if refName == "" {
			refName = strings.TrimSpace(in.Branch)
		}
		items, err := service.Search(ctx, in.ID, refName, in.Query, in.Path, in.Limit, in.MaxFiles, in.MaxFileSize, in.MatchCase, in.Regex)
		if err != nil {
			return nil, err
		}
		return &projectOutput{Body: items}, nil
	}

	createFileCommit := func(ctx context.Context, in *createFileCommitInput) (*projectOutput, error) {
		branchName := strings.TrimSpace(in.Body.BranchName)
		if err := service.CreateFileCommit(ctx, in.ID, projectservice.CreateFileCommitInput{
			BranchName:  branchName,
			Path:        in.Body.Path,
			Content:     in.Body.Content,
			Message:     in.Body.Message,
			AuthorName:  in.Body.AuthorName,
			AuthorEmail: in.Body.AuthorEmail,
		}); err != nil {
			return nil, err
		}
		body := map[string]any{"status": "created"}
		e.attachPipelineTrigger(ctx, body, in.ID, branchName)
		return &projectOutput{Body: body}, nil
	}

	languages := func(ctx context.Context, in *projectRepositoryInput) (*projectOutput, error) {
		branchName := strings.TrimSpace(in.Branch)
		if branchName == "" {
			branchName = strings.TrimSpace(in.Ref)
		}
		analysis, err := service.AnalyzeLanguages(ctx, in.ID, branchName)
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

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects", listProjects),
		httpapi.Get("/repos", listProjects, httpapi.DeprecatedRoute[projectsInput, projectOutput]("Use GET /projects instead.")),
		httpapi.Get("/projects/{id}", getProject),
		httpapi.Get("/repos/{id}", getProject, httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id} instead.")),
		httpapi.Post("/projects", createProject, httpapi.RequireUserRoute[createProjectInput, projectOutput](e.authRuntime)),
		httpapi.Post("/repos", createProject,
			httpapi.RequireUserRoute[createProjectInput, projectOutput](e.authRuntime),
			httpapi.DeprecatedRoute[createProjectInput, projectOutput]("Use POST /projects instead."),
		),
		httpapi.Delete("/projects/{id}", deleteProject, httpapi.RequireProjectWriteRoute[projectByIDInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Delete("/repos/{id}", deleteProject,
			httpapi.RequireProjectWriteRoute[projectByIDInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use DELETE /projects/{id} instead."),
		),
		httpapi.Get("/projects/{id}/repository/branches", listBranches),
		httpapi.Get("/repos/{id}/branches", listBranches, httpapi.DeprecatedRoute[projectByIDInput, projectOutput]("Use GET /projects/{id}/repository/branches instead.")),
		httpapi.Post("/projects/{id}/repository/branches", createBranch, httpapi.RequireProjectWriteRoute[createBranchInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/branches", createBranch,
			httpapi.RequireProjectWriteRoute[createBranchInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createBranchInput, projectOutput]("Use POST /projects/{id}/repository/branches instead."),
		),
		httpapi.Post("/projects/{id}/repository/branches/{branch_name}/protect", protectBranch, httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/projects/{id}/repository/branches/{branch_name}/unprotect", unprotectBranch, httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/branches/{branch_name}/protect", protectBranch,
			httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use POST /projects/{id}/repository/branches/{branch_name}/protect instead."),
		),
		httpapi.Post("/repos/{id}/branches/{branch_name}/unprotect", unprotectBranch,
			httpapi.RequireProjectWriteRoute[branchProtectionInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[branchProtectionInput, projectOutput]("Use POST /projects/{id}/repository/branches/{branch_name}/unprotect instead."),
		),
		httpapi.Get("/projects/{id}/repository/commits", listCommits),
		httpapi.Get("/repos/{id}/commits", listCommits, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/commits instead.")),
		httpapi.Get("/projects/{id}/repository/tree", listTree),
		httpapi.Get("/repos/{id}/tree", listTree, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/tree instead.")),
		httpapi.Get("/projects/{id}/repository/blob", getBlob),
		httpapi.Get("/repos/{id}/blob", getBlob, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/blob instead.")),
		httpapi.Get("/projects/{id}/repository/readme", getReadme),
		httpapi.Get("/repos/{id}/readme", getReadme, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/repository/readme instead.")),
		httpapi.Get("/projects/{id}/repository/search", searchRepository),
		httpapi.Get("/repos/{id}/search", searchRepository, httpapi.DeprecatedRoute[projectRepositorySearchInput, projectOutput]("Use GET /projects/{id}/repository/search instead.")),
		httpapi.Post("/projects/{id}/repository/files", createFileCommit, httpapi.RequireProjectWriteRoute[createFileCommitInput, projectOutput](e.authRuntime, projectWrite)),
		httpapi.Post("/projects/{id}/file-commits", createFileCommit,
			httpapi.RequireProjectWriteRoute[createFileCommitInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createFileCommitInput, projectOutput]("Use POST /projects/{id}/repository/files instead."),
		),
		httpapi.Post("/repos/{id}/file-commits", createFileCommit,
			httpapi.RequireProjectWriteRoute[createFileCommitInput, projectOutput](e.authRuntime, projectWrite),
			httpapi.DeprecatedRoute[createFileCommitInput, projectOutput]("Use POST /projects/{id}/repository/files instead."),
		),
		httpapi.Get("/projects/{id}/languages", languages),
		httpapi.Get("/repos/{id}/languages", languages, httpapi.DeprecatedRoute[projectRepositoryInput, projectOutput]("Use GET /projects/{id}/languages instead.")),
	)
}
