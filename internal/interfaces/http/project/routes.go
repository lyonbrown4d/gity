package project

import (
	"context"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionlist "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
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

type createProjectInput struct {
	Authorization string            `header:"Authorization"`
	Body          createProjectBody `json:"body"`
}

type projectByIDInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type projectsInput struct {
	OrganizationID int64  `query:"organization_id"`
	NamespaceID    int64  `query:"namespace_id"`
	IDs            string `query:"ids"`
}

type projectRepositoryInput struct {
	ID     int64  `path:"id"`
	Ref    string `query:"ref"`
	Branch string `query:"branch_name"`
	Path   string `query:"path"`
	Limit  int    `query:"limit"`
}

type createBranchInput struct {
	ID            int64            `path:"id"`
	Authorization string           `header:"Authorization"`
	Body          createBranchBody `json:"body"`
}

type branchProtectionInput struct {
	ID            int64  `path:"id"`
	BranchName    string `path:"branch_name"`
	Authorization string `header:"Authorization"`
}

type createFileCommitInput struct {
	ID            int64                `path:"id"`
	Authorization string               `header:"Authorization"`
	Body          createFileCommitBody `json:"body"`
}

type projectRepositorySearchInput struct {
	ID          int64  `path:"id"`
	Ref         string `query:"ref"`
	Branch      string `query:"branch_name"`
	Query       string `query:"query"`
	Path        string `query:"path"`
	Limit       int    `query:"limit"`
	MaxFiles    int    `query:"max_files"`
	MaxFileSize int64  `query:"max_file_size"`
	MatchCase   bool   `query:"match_case"`
	Regex       bool   `query:"regex"`
}

type projectOutput struct {
	Body any `json:"body"`
}

type createProjectBody struct {
	OrganizationID int64  `json:"organization_id"`
	NamespaceID    int64  `json:"namespace_id"`
	Key            string `json:"key"`
	Name           string `json:"name"`
	PathKey        string `json:"path_key"`
	Visibility     string `json:"visibility"`
	Description    string `json:"description"`
	DefaultBranch  string `json:"default_branch"`
}

type createBranchBody struct {
	Name      string `json:"name"`
	SourceRef string `json:"source_ref"`
}

type createFileCommitBody struct {
	BranchName  string `json:"branch_name"`
	Path        string `json:"path"`
	Content     string `json:"content"`
	Message     string `json:"message"`
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
}

type repositoryView struct {
	ID             string `json:"id"`
	UUID           string `json:"uuid"`
	OrganizationID string `json:"organization_id"`
	Key            string `json:"key"`
	Name           string `json:"name"`
	Description    string `json:"description"`
	Visibility     string `json:"visibility"`
	DefaultBranch  string `json:"default_branch"`
	CloneHTTPURL   string `json:"clone_http_url"`
}

type repositoryBranchView struct {
	RepositoryID  string `json:"repository_id"`
	Name          string `json:"name"`
	IsProtected   bool   `json:"is_protected"`
	LastCommitSHA string `json:"last_commit_sha,omitempty"`
}

type repositoryCommitView struct {
	RepositoryID string `json:"repository_id"`
	BranchName   string `json:"branch_name"`
	CommitSHA    string `json:"commit_sha"`
	Message      string `json:"message"`
	AuthorUserID string `json:"author_user_id"`
	CreatedAt    string `json:"created_at"`
}

type repositoryTreeEntryView struct {
	Name string `json:"name"`
	Path string `json:"path"`
	Kind string `json:"kind"`
	OID  string `json:"oid"`
	Size int64  `json:"size,omitempty"`
}

type repositoryBlobView struct {
	Path     string `json:"path"`
	Content  string `json:"content"`
	Size     int64  `json:"size"`
	IsBinary bool   `json:"is_binary"`
	Encoding string `json:"encoding"`
}

type repositoryLanguagesView struct {
	RepositoryID string                 `json:"repository_id"`
	BranchName   string                 `json:"branch_name"`
	Status       string                 `json:"status"`
	Revision     string                 `json:"revision,omitempty"`
	AnalyzedAt   string                 `json:"analyzed_at,omitempty"`
	TotalBytes   int64                  `json:"total_bytes"`
	Languages    []gitrepo.LanguageStat `json:"languages"`
}

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
		if e.pipelineService != nil {
			var branch projectservice.Branch
			var err error
			if branchName != "" {
				branch, err = service.GetBranch(ctx, in.ID, branchName)
			} else {
				branches, listErr := service.ListBranches(ctx, in.ID)
				err = listErr
				if err == nil {
					for _, item := range branches {
						if item.IsDefault {
							branch = item
							break
						}
					}
				}
			}
			if err == nil && branch.LastCommitSHA != "" {
				view, created, triggerErr := e.pipelineService.CreatePushPipeline(ctx, in.ID, branch.Name, branch.LastCommitSHA)
				if triggerErr != nil {
					body["pipeline_error"] = triggerErr.Error()
				} else if view.Pipeline.ID != 0 {
					body["pipeline_id"] = view.Pipeline.ID
					body["pipeline_created"] = created
				}
			}
		}
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

func (e *Endpoint) projectScope(ctx context.Context, projectID int64) (infraauth.ProjectScope, error) {
	item, err := e.service.GetByID(ctx, projectID)
	if err != nil {
		return infraauth.ProjectScope{}, err
	}
	return infraauth.ProjectScope{ID: item.ID, NamespaceID: item.NamespaceID, Visibility: item.Visibility}, nil
}

func toRepositoryView(item projectdomain.Project, settings config.Settings) repositoryView {
	baseURL := strings.TrimRight(settings.HTTP.BaseURL, "/")
	return repositoryView{
		ID:             strconv.FormatInt(item.ID, 10),
		UUID:           strconv.FormatInt(item.ID, 10),
		OrganizationID: strconv.FormatInt(item.NamespaceID, 10),
		Key:            item.PathKey,
		Name:           item.Name,
		Description:    item.Description,
		Visibility:     item.Visibility,
		DefaultBranch:  item.DefaultBranch,
		CloneHTTPURL:   baseURL + "/" + strings.Trim(item.FullPath, "/") + ".git",
	}
}

func toRepositoryBranchView(projectID int64, item projectservice.Branch) repositoryBranchView {
	return repositoryBranchView{
		RepositoryID:  strconv.FormatInt(projectID, 10),
		Name:          item.Name,
		IsProtected:   item.IsProtected,
		LastCommitSHA: item.LastCommitSHA,
	}
}

func toRepositoryCommitView(projectID int64, branchName string, item gitrepo.Commit) repositoryCommitView {
	return repositoryCommitView{
		RepositoryID: strconv.FormatInt(projectID, 10),
		BranchName:   branchName,
		CommitSHA:    item.Hash,
		Message:      item.Message,
		AuthorUserID: item.AuthorName,
		CreatedAt:    item.CommittedAt,
	}
}

func toRepositoryTreeEntryView(item gitrepo.TreeEntry) repositoryTreeEntryView {
	return repositoryTreeEntryView{
		Name: item.Name,
		Path: item.Path,
		Kind: item.Type,
		OID:  item.Mode,
		Size: item.Size,
	}
}

func toRepositoryBlobView(item gitrepo.Blob) repositoryBlobView {
	return repositoryBlobView{
		Path:     item.Path,
		Content:  item.Content,
		Size:     item.Size,
		IsBinary: item.Encoding == "base64",
		Encoding: item.Encoding,
	}
}

func (in createProjectInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectByIDInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectByIDInput) ProjectIDValue() int64 {
	return in.ID
}

func (in createBranchInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createBranchInput) ProjectIDValue() int64 {
	return in.ID
}

func (in branchProtectionInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in branchProtectionInput) ProjectIDValue() int64 {
	return in.ID
}

func (in createFileCommitInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createFileCommitInput) ProjectIDValue() int64 {
	return in.ID
}

func parseIDFilter(raw string) *setx.Set[int64] {
	ids := setx.NewSet[int64]()
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ids
	}
	for _, part := range strings.Split(raw, ",") {
		id, err := strconv.ParseInt(strings.TrimSpace(part), 10, 64)
		if err == nil && id > 0 {
			ids.Add(id)
		}
	}
	return ids
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return value
		}
	}
	return ""
}
