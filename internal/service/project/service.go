package project

import (
	"log/slog"

	"github.com/DaiYuANg/arcgo/dbx"
)

type Service struct {
	logger *slog.Logger
	db     *dbx.DB
}

func NewService(logger *slog.Logger, db *dbx.DB) *Service {
	return &Service{logger: logger, db: db}
}
