package system

import (
	"context"

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

type healthOutput struct {
	Body struct {
		Status      string `json:"status"`
		Application string `json:"application"`
		Stack       string `json:"stack"`
	} `json:"body"`
}

type rewriteInfoOutput struct {
	Body struct {
		Runtime      string   `json:"runtime"`
		Architecture string   `json:"architecture"`
		Foundations  []string `json:"foundations"`
	} `json:"body"`
}

type Endpoint struct {
	settings config.Settings
}

func NewEndpoint(settings config.Settings) *Endpoint {
	return &Endpoint{settings: settings}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("", "System", "System", "System and runtime APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	settings := e.settings

	health := func(ctx context.Context, in *struct{}) (*healthOutput, error) {
		_ = ctx
		_ = in
		out := &healthOutput{}
		out.Body.Status = "ok"
		out.Body.Application = settings.App.Name
		out.Body.Stack = "go"
		return out, nil
	}

	rewriteInfo := func(ctx context.Context, in *struct{}) (*rewriteInfoOutput, error) {
		_ = ctx
		_ = in
		out := &rewriteInfoOutput{}
		out.Body.Runtime = "go"
		out.Body.Architecture = "single-module cmd+internal monorepo"
		out.Body.Foundations = []string{
			"arcgolabs/dix",
			"arcgolabs/httpx",
			"arcgolabs/authx",
			"arcgolabs/dbx",
			"go-git + native git",
		}
		return out, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/health", health),
		httpapi.Get("/v1/rewrite/info", rewriteInfo),
	)
}
