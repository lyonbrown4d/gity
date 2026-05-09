package lfs

import (
	"context"

	lfsservice "github.com/DaiYuANg/gity/internal/application/lfs"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

type projectLFSObjectsInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
	Cursor        string `query:"cursor"`
	Limit         int    `query:"limit"`
}

type projectLFSLocksInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
	Path          string `query:"path"`
	ID            string `query:"id"`
	Cursor        string `query:"cursor"`
	Limit         int    `query:"limit"`
}

type createProjectLFSLockInput struct {
	ProjectID     int64                      `path:"id"`
	Authorization string                     `header:"Authorization"`
	Body          lfsservice.CreateLockInput `json:"body"`
}

type unlockProjectLFSLockInput struct {
	ProjectID     int64                  `path:"id"`
	LockID        string                 `path:"lock_id"`
	Authorization string                 `header:"Authorization"`
	Body          lfsservice.UnlockInput `json:"body"`
}

type projectLFSOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service        *lfsservice.Service
	projectService *projectservice.Service
	authRuntime    *infraauth.Runtime
}

func NewEndpoint(service *lfsservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Git LFS", "Git LFS", "Project Git LFS management APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/lfs/objects", e.listObjects, httpapi.Policy(httpapi.RequireProjectRead[projectLFSObjectsInput, projectLFSOutput](authRuntime, projectScope))),
		httpapi.Get("/projects/{id}/lfs/locks", e.listLocks, httpapi.Policy(httpapi.RequireProjectRead[projectLFSLocksInput, projectLFSOutput](authRuntime, projectScope))),
		httpapi.Post("/projects/{id}/lfs/locks", e.createLock, httpapi.RequireProjectWriteRoute[createProjectLFSLockInput, projectLFSOutput](authRuntime, projectScope)),
		httpapi.Post("/projects/{id}/lfs/locks/{lock_id}/unlock", e.unlockLock, httpapi.RequireProjectWriteRoute[unlockProjectLFSLockInput, projectLFSOutput](authRuntime, projectScope)),
	)
}

func (e *Endpoint) listObjects(ctx context.Context, in *projectLFSObjectsInput) (*projectLFSOutput, error) {
	result, err := e.service.ListObjects(ctx, in.ProjectID, lfsservice.ObjectListInput{Cursor: in.Cursor, Limit: in.Limit})
	if err != nil {
		return nil, err
	}
	return &projectLFSOutput{Body: result}, nil
}

func (e *Endpoint) listLocks(ctx context.Context, in *projectLFSLocksInput) (*projectLFSOutput, error) {
	result, err := e.service.ListLocks(ctx, in.ProjectID, lfsservice.LockListInput{Path: in.Path, ID: in.ID, Cursor: in.Cursor, Limit: in.Limit})
	if err != nil {
		return nil, err
	}
	return &projectLFSOutput{Body: result}, nil
}

func (e *Endpoint) createLock(ctx context.Context, in *createProjectLFSLockInput) (*projectLFSOutput, error) {
	userID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, 0)
	if err != nil {
		return nil, err
	}
	result, err := e.service.CreateLock(ctx, in.ProjectID, userID, in.Body)
	if err != nil {
		return nil, err
	}
	return &projectLFSOutput{Body: result}, nil
}

func (e *Endpoint) unlockLock(ctx context.Context, in *unlockProjectLFSLockInput) (*projectLFSOutput, error) {
	userID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, 0)
	if err != nil {
		return nil, err
	}
	result, err := e.service.Unlock(ctx, in.ProjectID, userID, in.LockID, in.Body)
	if err != nil {
		return nil, err
	}
	return &projectLFSOutput{Body: result}, nil
}

func (in projectLFSObjectsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectLFSObjectsInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectLFSLocksInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectLFSLocksInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createProjectLFSLockInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createProjectLFSLockInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in unlockProjectLFSLockInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in unlockProjectLFSLockInput) ProjectIDValue() int64 {
	return in.ProjectID
}
