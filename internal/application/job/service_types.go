package job

import (
	"log/slog"
	"strings"
	"time"

	setx "github.com/arcgolabs/collectionx/set"
	"github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

const (
	KindNoop   = "noop"
	KindScript = "script"
)

var supportedKinds = setx.NewSet(KindNoop, KindScript)

type Service struct {
	logger       *slog.Logger
	projectRepo  ports.ProjectRepository
	jobRepo      ports.ProjectJobRepository
	logRepo      ports.ProjectJobLogRepository
	artifactRepo ports.ProjectJobArtifactRepository
	storage      ports.ObjectStorage
}

type CreateInput struct {
	Kind        string    `json:"kind"`
	Payload     string    `json:"payload"`
	MaxAttempts int       `json:"max_attempts"`
	RunAfter    time.Time `json:"run_after"`
}

type ClaimMatcher func(cidomain.ProjectJob) (bool, error)

type Dependencies struct {
	Logger       *slog.Logger
	ProjectRepo  ports.ProjectRepository
	JobRepo      ports.ProjectJobRepository
	LogRepo      ports.ProjectJobLogRepository
	ArtifactRepo ports.ProjectJobArtifactRepository
	Storage      ports.ObjectStorage
}

func NewDependencies(
	logger *slog.Logger,
	projectRepo ports.ProjectRepository,
	jobRepo ports.ProjectJobRepository,
	logRepo ports.ProjectJobLogRepository,
	artifactRepo ports.ProjectJobArtifactRepository,
	storage ports.ObjectStorage,
) Dependencies {
	return Dependencies{Logger: logger, ProjectRepo: projectRepo, JobRepo: jobRepo, LogRepo: logRepo, ArtifactRepo: artifactRepo, Storage: storage}
}

func NewServiceWithDependencies(dependencies Dependencies) *Service {
	return &Service{logger: dependencies.Logger, projectRepo: dependencies.ProjectRepo, jobRepo: dependencies.JobRepo, logRepo: dependencies.LogRepo, artifactRepo: dependencies.ArtifactRepo, storage: dependencies.Storage}
}

func NewService(
	logger *slog.Logger,
	projectRepo ports.ProjectRepository,
	jobRepo ports.ProjectJobRepository,
	logRepo ports.ProjectJobLogRepository,
	artifactRepo ports.ProjectJobArtifactRepository,
	storage ports.ObjectStorage,
) *Service {
	return NewServiceWithDependencies(NewDependencies(logger, projectRepo, jobRepo, logRepo, artifactRepo, storage))
}

func retryDelay(attempts int) time.Duration {
	if attempts <= 0 {
		return 5 * time.Second
	}
	delay := time.Duration(attempts*5) * time.Second
	if delay > time.Minute {
		return time.Minute
	}
	return delay
}

func normalizeWorkerID(workerID string) string {
	trimmed := strings.TrimSpace(workerID)
	if trimmed == "" {
		return "worker"
	}
	return trimmed
}
