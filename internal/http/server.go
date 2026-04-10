package http

import (
	"context"
	"fmt"
	"log/slog"
	"time"

	"github.com/DaiYuANg/arcgo/httpx"
	"github.com/DaiYuANg/arcgo/httpx/adapter"
	httpxstd "github.com/DaiYuANg/arcgo/httpx/adapter/std"
	"github.com/DaiYuANg/gity/internal/config"
)

type Host struct {
	server  httpx.ServerRuntime
	address string
	logger  *slog.Logger
	cancel  context.CancelFunc
	done    chan error
}

func NewServer(settings config.Settings, logger *slog.Logger) (httpx.ServerRuntime, error) {
	adapterRuntime := httpxstd.New(nil, adapter.HumaOptions{
		Title:       settings.App.Name,
		Version:     "0.1.0",
		Description: "Gity Go rewrite API on arcgo/httpx",
		DocsPath:    "/docs",
		OpenAPIPath: "/openapi.json",
	})

	return httpx.New(
		httpx.WithAdapter(adapterRuntime),
		httpx.WithLogger(logger),
		httpx.WithBasePath("/api"),
		httpx.WithValidation(),
	), nil
}

func NewHost(server httpx.ServerRuntime, settings config.Settings, logger *slog.Logger) *Host {
	return &Host{
		server:  server,
		address: settings.HTTP.Address,
		logger:  logger,
		done:    make(chan error, 1),
	}
}

func (h *Host) Start(context.Context) error {
	if h == nil || h.server == nil {
		return nil
	}

	runCtx, cancel := context.WithCancel(context.Background())
	h.cancel = cancel
	go func() {
		h.done <- h.server.ListenAndServeContext(runCtx, h.address)
	}()

	select {
	case err := <-h.done:
		return err
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
		return ctx.Err()
	case <-time.After(5 * time.Second):
	}

	h.logger.Info("http server stopped", slog.String("address", h.address))
	return nil
}

func DocsURL(settings config.Settings) string {
	return fmt.Sprintf("%s/docs", settings.HTTP.BaseURL)
}
