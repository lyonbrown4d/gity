package searchindex

import "log/slog"

func (s *Service) logInfo(message string, attrs ...slog.Attr) {
	if s.logger == nil {
		return
	}
	s.logger.Info(message, attrsToAny(attrs)...)
}

func (s *Service) logError(message string, err error, attrs ...slog.Attr) {
	if s.logger == nil {
		return
	}
	args := append([]slog.Attr{slog.String("error", err.Error())}, attrs...)
	s.logger.Error(message, attrsToAny(args)...)
}

func attrsToAny(attrs []slog.Attr) []any {
	out := make([]any, 0, len(attrs))
	for _, attr := range attrs {
		out = append(out, attr)
	}
	return out
}

func slogProjectID(projectID int64) slog.Attr {
	return slog.Int64("project_id", projectID)
}

func slogIndexPath(indexPath string) slog.Attr {
	return slog.String("index_path", indexPath)
}

func slogRevision(revision string) slog.Attr {
	return slog.String("revision", revision)
}

func slogIndexedFiles(indexedFiles int) slog.Attr {
	return slog.Int("files", indexedFiles)
}
