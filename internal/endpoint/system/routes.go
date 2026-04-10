package system

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	"github.com/DaiYuANg/gity/internal/config"
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

func RegisterRoutes(server httpx.ServerRuntime, settings config.Settings) {
	httpx.MustGet(server, "/health", func(ctx context.Context, in *struct{}) (*healthOutput, error) {
		_ = ctx
		_ = in
		out := &healthOutput{}
		out.Body.Status = "ok"
		out.Body.Application = settings.App.Name
		out.Body.Stack = "go"
		return out, nil
	})

	v1 := server.Group("/v1")
	httpx.MustGroupGet(v1, "/rewrite/info", func(ctx context.Context, in *struct{}) (*rewriteInfoOutput, error) {
		_ = ctx
		_ = in
		out := &rewriteInfoOutput{}
		out.Body.Runtime = "go"
		out.Body.Architecture = "single-module cmd+internal monorepo"
		out.Body.Foundations = []string{
			"arcgo/dix",
			"arcgo/httpx",
			"arcgo/authx",
			"arcgo/dbx",
			"go-git + native git",
		}
		return out, nil
	})
}
