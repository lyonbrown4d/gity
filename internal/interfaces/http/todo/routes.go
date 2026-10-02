package todo

import (
	"github.com/arcgolabs/httpx"
	todoservice "github.com/lyonbrown4d/gity/internal/application/todo"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type todosInput struct {
	Authorization string `header:"Authorization"`
	State         string `query:"state"`
	Limit         int    `query:"limit"`
}

type todoByIDInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type todoOutput struct {
	Body any `json:"body"`
}

type Endpoint struct {
	service     *todoservice.Service
	authRuntime *infraauth.Runtime
}

func NewEndpoint(service *todoservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Todos", "Todos", "User todo APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/todos", e.listTodos, httpapi.RequireUserRoute[todosInput, todoOutput](e.authRuntime)),
		httpapi.Post("/todos/{id}/done", e.markTodoDone, httpapi.RequireUserRoute[todoByIDInput, todoOutput](e.authRuntime)),
	)
}

func (in todosInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in todoByIDInput) AuthorizationHeader() string {
	return in.Authorization
}
