package logger

import (
	"log/slog"

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/arcgolabs/logx"
)

func NewLogger(settings config.Settings) (*slog.Logger, error) {
	if settings.App.Environment == "production" {
		return logx.NewProduction()
	}
	return logx.NewDevelopment()
}
