package httpserver

import (
	"context"
	"log/slog"
	"strings"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/httpx/adapter"
	httpxfiber "github.com/arcgolabs/httpx/adapter/fiber"
	"github.com/gofiber/fiber/v3"
	"github.com/gofiber/fiber/v3/middleware/cors"
	"github.com/lyonbrown4d/gity/internal/config"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
	"github.com/samber/oops"
)

type Host struct {
	server  httpx.ServerRuntime
	address string
	logger  *slog.Logger
	cancel  context.CancelFunc
	done    chan error
}

func NewFiberApp() *fiber.App {
	return fiber.New()
}

func NewServer(app *fiber.App, settings config.Settings, logger *slog.Logger, endpoints *collectionlist.List[httpx.Endpoint]) (httpx.ServerRuntime, error) {
	if strings.EqualFold(settings.App.Environment, "development") {
		app.Use(cors.New(cors.Config{
			AllowOrigins: []string{"*"},
			AllowHeaders: []string{"*"},
		}))
	}

	adapterRuntime := httpxfiber.New(app, adapter.HumaOptions{
		Title:       settings.App.Name,
		Version:     "0.1.0",
		Description: "Gity Go rewrite API on arcgolabs/httpx fiber",
		DocsPath:    "/docs",
		OpenAPIPath: "/openapi.json",
	})

	server := httpx.New(
		httpx.WithAdapter(adapterRuntime),
		httpx.WithLogger(logger),
		httpx.WithBasePath("/api"),
		httpx.WithValidation(),
	)
	httpapi.Configure(server)
	httpapi.RegisterEndpoints(server, endpoints)
	return server, nil
}

func NewHost(server httpx.ServerRuntime, settings config.Settings, logger *slog.Logger) *Host {
	return &Host{
		server:  server,
		address: settings.HTTP.Address,
		logger:  logger,
		done:    make(chan error, 1),
	}
}

func (h *Host) Start(ctx context.Context) error {
	if h == nil || h.server == nil {
		return nil
	}

	// The hook context is canceled when OnStart returns; keep the listener alive until Stop cancels it.
	runCtx, cancel := context.WithCancel(context.Background())
	h.cancel = cancel
	go func() {
		h.done <- h.server.ListenAndServeContext(runCtx, h.address)
	}()

	select {
	case err := <-h.done:
		return err
	case <-ctx.Done():
		cancel()
		return oops.In("http_server").Wrap(ctx.Err())
	case <-time.After(200 * time.Millisecond):
	}

	h.logger.Info("http server started", slog.String("address", h.address))
	return nil
}

func (h *Host) Stop(ctx context.Context) error {
	if h == nil {
		return nil
	}
	if h.cancel != nil {
		h.cancel()
	}

	select {
	case err := <-h.done:
		if err != nil {
			h.logger.Info("http server stopped", slog.String("address", h.address), slog.String("error", err.Error()))
			return nil
		}
	case <-ctx.Done():
		return oops.In("http_server").Wrap(ctx.Err())
	case <-time.After(5 * time.Second):
	}

	h.logger.Info("http server stopped", slog.String("address", h.address))
	return nil
}

func DocsURL(settings config.Settings) string {
	return settings.HTTP.BaseURL + "/docs"
}
