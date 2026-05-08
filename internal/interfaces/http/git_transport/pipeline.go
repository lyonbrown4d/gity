package gittransport

import (
	"context"
	"log/slog"

	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
)

func triggerPushPipelines(ctx context.Context, logger *slog.Logger, service *pipelineservice.Service, project projectView, updates []receivePackUpdate) {
	if service == nil {
		return
	}
	for _, update := range updates {
		if update.BranchName == "" || update.Delete || isZeroOID(update.NewSHA) {
			continue
		}
		view, created, err := service.CreatePushPipeline(ctx, project.ID, update.BranchName, update.NewSHA)
		if err != nil {
			logger.Warn("create push pipeline failed", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.String("error", err.Error()))
			continue
		}
		if view.Pipeline.ID == 0 {
			continue
		}
		if created {
			logger.Info("push pipeline created", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.Int64("pipeline_id", view.Pipeline.ID))
		} else {
			logger.Debug("push pipeline already exists", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.Int64("pipeline_id", view.Pipeline.ID))
		}
	}
}
