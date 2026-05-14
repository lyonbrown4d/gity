// Package logger configures structured logging.
package logger

import (
	"log/slog"

	"github.com/arcgolabs/logx"
	"github.com/lyonbrown4d/gity/internal/config"
	"github.com/samber/oops"
)

func NewLogger(settings config.Settings) (*slog.Logger, error) {
	if settings.App.Environment == "production" {
		logger, err := logx.NewProduction()
		if err != nil {
			return nil, oops.In("logger").With("environment", settings.App.Environment).Wrapf(err, "initialize production logger")
		}
		return logger, nil
	}
	logger, err := logx.NewDevelopment()
	if err != nil {
		return nil, oops.In("logger").With("environment", settings.App.Environment).Wrapf(err, "initialize development logger")
	}
	return logger, nil
}
