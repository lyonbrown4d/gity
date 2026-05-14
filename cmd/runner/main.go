// Package main contains the gity runner command.
package main

import (
	"context"
	"errors"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/arcgolabs/logx"
	"github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
	"github.com/samber/oops"
)

func main() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}

func run() error {
	cfg, err := runneragent.ConfigFromEnv(os.Args[1:])
	if err != nil {
		return oops.In("runner").Wrapf(err, "load runner config")
	}

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	logger, err := logx.NewDevelopment()
	if err != nil {
		return oops.In("runner").Wrapf(err, "initialize runner logger")
	}
	agent := runneragent.New(cfg, logger)
	if cfg.Once {
		if _, err := agent.RunOnce(ctx); err != nil && !errors.Is(err, context.Canceled) {
			return oops.In("runner").Wrapf(err, "run runner once")
		}
		return nil
	}
	if err := agent.Run(ctx); err != nil && !errors.Is(err, context.Canceled) {
		return oops.In("runner").Wrapf(err, "run runner")
	}
	return nil
}
