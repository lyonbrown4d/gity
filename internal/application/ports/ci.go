package ports

import (
	"context"
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	collectionx "github.com/arcgolabs/collectionx/list"
)

const (
	ProjectJobStatusPending   = "pending"
	ProjectJobStatusRunning   = "running"
	ProjectJobStatusSucceeded = "succeeded"
	ProjectJobStatusFailed    = "failed"
	ProjectJobStatusCancelled = "cancelled"

	ProjectPipelineStatusPending   = "pending"
	ProjectPipelineStatusRunning   = "running"
	ProjectPipelineStatusSucceeded = "succeeded"
	ProjectPipelineStatusFailed    = "failed"
	ProjectPipelineStatusCancelled = "cancelled"

	ProjectRunnerStatusOnline  = "online"
	ProjectRunnerStatusOffline = "offline"
)

type ProjectJobRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectJob], error)
	GetByProjectAndID(ctx context.Context, projectID int64, id int64) (cidomain.ProjectJob, error)
	GetByID(ctx context.Context, id int64) (cidomain.ProjectJob, error)
	Create(ctx context.Context, input CreateProjectJobInput) (cidomain.ProjectJob, error)
	ClaimNext(ctx context.Context, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error)
	ClaimNextByKinds(ctx context.Context, kinds []string, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error)
	ClaimNextByProjectID(ctx context.Context, projectID int64, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error)
	ClaimNextByProjectIDAndKinds(ctx context.Context, projectID int64, kinds []string, workerID string, lease time.Duration) (cidomain.ProjectJob, bool, error)
	MarkSucceeded(ctx context.Context, id int64, result string) error
	ScheduleByID(ctx context.Context, id int64, runAfter time.Time) error
	MarkFailed(ctx context.Context, item cidomain.ProjectJob, message string, retryAfter time.Duration) error
	CancelByID(ctx context.Context, id int64) error
	RetryByID(ctx context.Context, id int64, runAfter time.Time) error
}

type ProjectJobArtifactRepository interface {
	ListByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (*collectionx.List[cidomain.ProjectJobArtifact], error)
	GetByProjectJobAndID(ctx context.Context, projectID int64, projectJobID int64, artifactID int64) (cidomain.ProjectJobArtifact, error)
	GetByID(ctx context.Context, id int64) (cidomain.ProjectJobArtifact, error)
	Create(ctx context.Context, input CreateProjectJobArtifactInput) (cidomain.ProjectJobArtifact, error)
	MarkStored(ctx context.Context, id int64, input StoreProjectJobArtifactInput) error
	DeleteByID(ctx context.Context, id int64) error
}

type ProjectJobLogRepository interface {
	ListByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (*collectionx.List[cidomain.ProjectJobLog], error)
	LatestByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (cidomain.ProjectJobLog, error)
	GetByProjectJobAttempt(ctx context.Context, projectID int64, projectJobID int64, attempt int) (cidomain.ProjectJobLog, error)
	Create(ctx context.Context, input CreateProjectJobLogInput) (cidomain.ProjectJobLog, error)
	Append(ctx context.Context, input AppendProjectJobLogInput) (cidomain.ProjectJobLog, error)
	UpsertAttempt(ctx context.Context, input CreateProjectJobLogInput) (cidomain.ProjectJobLog, error)
}

type ProjectPipelineRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectPipeline], error)
	GetByProjectAndID(ctx context.Context, projectID int64, id int64) (cidomain.ProjectPipeline, error)
	GetByProjectSourceRefCommit(ctx context.Context, projectID int64, source string, refName string, commitSHA string) (cidomain.ProjectPipeline, error)
	GetLatestByProjectRefCommit(ctx context.Context, projectID int64, refName string, commitSHA string) (cidomain.ProjectPipeline, error)
	Create(ctx context.Context, input CreateProjectPipelineInput) (cidomain.ProjectPipeline, error)
	UpdateStatus(ctx context.Context, item cidomain.ProjectPipeline, status string) error
}

type ProjectPipelineJobRepository interface {
	ListByPipelineID(ctx context.Context, projectID int64, pipelineID int64) (*collectionx.List[cidomain.ProjectPipelineJob], error)
	GetByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (cidomain.ProjectPipelineJob, error)
	Create(ctx context.Context, input CreateProjectPipelineJobInput) (cidomain.ProjectPipelineJob, error)
}

type ProjectRunnerRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectRunner], error)
	GetByProjectAndID(ctx context.Context, projectID int64, id int64) (cidomain.ProjectRunner, error)
	GetByToken(ctx context.Context, token string) (cidomain.ProjectRunner, error)
	Create(ctx context.Context, input CreateProjectRunnerInput) (cidomain.ProjectRunner, error)
	MarkHeartbeat(ctx context.Context, id int64) error
	DeleteByID(ctx context.Context, id int64) error
}

type CreateProjectJobInput struct {
	ProjectID   int64
	Kind        string
	Payload     string
	MaxAttempts int
	RunAfter    time.Time
}

type CreateProjectJobArtifactInput struct {
	ProjectID    int64
	ProjectJobID int64
	Name         string
	FileName     string
	FilePath     string
	ContentType  string
}

type StoreProjectJobArtifactInput struct {
	ContentType string
	ByteSize    int64
	StorageKey  string
}

type CreateProjectJobLogInput struct {
	ProjectID       int64
	ProjectJobID    int64
	Attempt         int
	ExitCode        int
	Output          string
	OutputTruncated bool
	DurationMillis  int64
}

type AppendProjectJobLogInput struct {
	ProjectID       int64
	ProjectJobID    int64
	Attempt         int
	Output          string
	OutputTruncated bool
	DurationMillis  int64
}

type CreateProjectPipelineInput struct {
	ProjectID     int64
	Name          string
	Source        string
	RefName       string
	CommitSHA     string
	Status        string
	ConfigSource  string
	ConfigContent string
}

type CreateProjectPipelineJobInput struct {
	ProjectID    int64
	PipelineID   int64
	ProjectJobID int64
	Name         string
	Stage        string
	Needs        string
	Image        string
	Script       string
	Artifacts    string
	SortOrder    int
}

type CreateProjectRunnerInput struct {
	ProjectID   int64
	Name        string
	Description string
	Tags        string
	Token       string
}
