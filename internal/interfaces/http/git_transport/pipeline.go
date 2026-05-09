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
		triggerPushPipeline(ctx, logger, service, project, update)
	}
}

func triggerPushPipeline(ctx context.Context, logger *slog.Logger, service *pipelineservice.Service, project projectView, update receivePackUpdate) {
	if !shouldTriggerPushPipeline(update) {
		return
	}
	view, created, err := service.CreatePushPipeline(ctx, project.ID, update.BranchName, update.NewSHA)
	if err != nil {
		logger.Warn("create push pipeline failed", slog.String("project", project.FullPath), slog.String("branch", update.BranchName), slog.String("commit", update.NewSHA), slog.String("error", err.Error()))
		return
	}
	if view.Pipeline.ID == 0 {
		return
	}
	logPushPipeline(logger, project, update, view.Pipeline.ID, created)
}

func shouldTriggerPushPipeline(update receivePackUpdate) bool {
	return update.BranchName != "" && !update.Delete && !isZeroOID(update.NewSHA)
}

func logPushPipeline(logger *slog.Logger, project projectView, update receivePackUpdate, pipelineID int64, created bool) {
	attrs := []slog.Attr{
		slog.String("project", project.FullPath),
		slog.String("branch", update.BranchName),
		slog.String("commit", update.NewSHA),
		slog.Int64("pipeline_id", pipelineID),
	}
	if created {
		logger.Info("push pipeline created", attrsToArgs(attrs)...)
		return
	}
	logger.Debug("push pipeline already exists", attrsToArgs(attrs)...)
}

func attrsToArgs(attrs []slog.Attr) []any {
	args := make([]any, 0, len(attrs))
	for _, attr := range attrs {
		args = append(args, attr)
	}
	return args
}
