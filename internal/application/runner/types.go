package runner

import (
	"time"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type RegisterInput struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Tags        string `json:"tags"`
}

type UploadArtifactInput struct {
	Name          string `json:"name"`
	FileName      string `json:"file_name"`
	FilePath      string `json:"file_path"`
	ContentType   string `json:"content_type"`
	ContentBase64 string `json:"content_base64"`
}

type AppendTraceInput struct {
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
}

type SourceArchiveView struct {
	FileName      string `json:"file_name"`
	Encoding      string `json:"encoding"`
	ContentBase64 string `json:"content_base64"`
}

type scriptSourcePayload struct {
	ProjectFullPath string `json:"project_full_path"`
	RefName         string `json:"ref_name"`
	CommitSHA       string `json:"commit_sha"`
}

type RunnerView struct {
	ID            int64      `json:"id"`
	ProjectID     int64      `json:"project_id"`
	Name          string     `json:"name"`
	Description   string     `json:"description"`
	Tags          string     `json:"tags"`
	Status        string     `json:"status"`
	Active        bool       `json:"active"`
	LastContactAt *time.Time `json:"last_contact_at,omitempty"`
	CreatedAt     time.Time  `json:"created_at"`
	UpdatedAt     time.Time  `json:"updated_at"`
}

type RegistrationView struct {
	Runner RunnerView `json:"runner"`
	Token  string     `json:"token"`
}

type ClaimView struct {
	Claimed bool                `json:"claimed"`
	Runner  RunnerView          `json:"runner"`
	Job     cidomain.ProjectJob `json:"job,omitzero"`
}
