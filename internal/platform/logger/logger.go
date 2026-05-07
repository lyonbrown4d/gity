package logger

import (
	"log/slog"

	"github.com/arcgolabs/logx"
	"github.com/DaiYuANg/gity/internal/config"
)

func NewLogger(settings config.Settings) (*slog.Logger, error) {
	if settings.App.Environment == "production" {
		return logx.NewProduction()
	}
	return logx.NewDevelopment()
}
