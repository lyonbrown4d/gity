// Package project defines project domain models and events.
package project

import domainevent "github.com/DaiYuANg/gity/internal/domain/event"

const (
	EventProjectCreated                 = "project.created"
	EventProjectDeleted                 = "project.deleted"
	EventProjectRepositoryChanged       = "project.repository.changed"
	EventProjectBranchProtectionChanged = "project.branch_protection.changed"
	EventProjectBranchDeleted           = "project.branch.deleted"
)

type ProjectCreated struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	OrganizationID int64  `json:"organization_id"`
	ProjectName    string `json:"name"`
	PathKey        string `json:"path_key"`
	FullPath       string `json:"full_path"`
	Visibility     string `json:"visibility"`
	DefaultBranch  string `json:"default_branch"`
}

func (ProjectCreated) Name() string {
	return EventProjectCreated
}

func NewProjectCreatedEvent(project Project) ProjectCreated {
	return ProjectCreated{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      project.ID,
		OrganizationID: project.OrganizationID,
		ProjectName:    project.Name,
		PathKey:        project.PathKey,
		FullPath:       project.FullPath,
		Visibility:     project.Visibility,
		DefaultBranch:  project.DefaultBranch,
	}
}

type ProjectDeleted struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	OrganizationID int64  `json:"organization_id"`
	ProjectName    string `json:"name"`
	PathKey        string `json:"path_key"`
	FullPath       string `json:"full_path"`
	Status         string `json:"status"`
	DeletedAt      string `json:"deleted_at,omitempty"`
}

func (ProjectDeleted) Name() string {
	return EventProjectDeleted
}

func NewProjectDeletedEvent(project Project) ProjectDeleted {
	deletedAt := ""
	if !project.DeletedAt.IsZero() {
		deletedAt = project.DeletedAt.UTC().Format("2006-01-02T15:04:05Z")
	}
	return ProjectDeleted{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      project.ID,
		OrganizationID: project.OrganizationID,
		ProjectName:    project.Name,
		PathKey:        project.PathKey,
		FullPath:       project.FullPath,
		Status:         project.Status,
		DeletedAt:      deletedAt,
	}
}

type ProjectRepositoryChanged struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	OrganizationID int64  `json:"organization_id"`
	FullPath       string `json:"full_path"`
	DefaultBranch  string `json:"default_branch"`
	BranchName     string `json:"branch_name"`
	CommitSHA      string `json:"commit_sha,omitempty"`
	Deleted        bool   `json:"deleted"`
	Source         string `json:"source,omitempty"`
}

func (ProjectRepositoryChanged) Name() string {
	return EventProjectRepositoryChanged
}

func NewProjectRepositoryChangedEvent(project Project, branchName, commitSHA string, deleted bool, source string) ProjectRepositoryChanged {
	return ProjectRepositoryChanged{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      project.ID,
		OrganizationID: project.OrganizationID,
		FullPath:       project.FullPath,
		DefaultBranch:  project.DefaultBranch,
		BranchName:     branchName,
		CommitSHA:      commitSHA,
		Deleted:        deleted,
		Source:         source,
	}
}

func (e ProjectRepositoryChanged) AffectsDefaultBranch() bool {
	if e.Deleted {
		return false
	}
	return e.BranchName == "" || e.BranchName == e.DefaultBranch || e.BranchName == "refs/heads/"+e.DefaultBranch
}

type ProjectBranchProtectionChanged struct {
	domainevent.Metadata
	ProjectID              int64  `json:"project_id"`
	BranchName             string `json:"branch_name"`
	Protected              bool   `json:"protected"`
	RuleType               string `json:"rule_type,omitempty"`
	PushAccessLevel        string `json:"push_access_level,omitempty"`
	MergeAccessLevel       string `json:"merge_access_level,omitempty"`
	RequireMergeRequest    bool   `json:"require_merge_request"`
	RequirePipelineSuccess bool   `json:"require_pipeline_success"`
	AllowForcePush         bool   `json:"allow_force_push"`
	AllowDelete            bool   `json:"allow_delete"`
}

func (ProjectBranchProtectionChanged) Name() string {
	return EventProjectBranchProtectionChanged
}

func NewProjectBranchProtectionChangedEvent(protection ProjectBranchProtection, protected bool) ProjectBranchProtectionChanged {
	return ProjectBranchProtectionChanged{
		Metadata:               domainevent.NewMetadata(),
		ProjectID:              protection.ProjectID,
		BranchName:             protection.BranchName,
		Protected:              protected,
		RuleType:               NormalizeProjectBranchProtectionRuleType(protection.RuleType, protection.BranchName),
		PushAccessLevel:        NormalizeProjectBranchProtectionAccessLevel(protection.PushAccessLevel, ProjectBranchProtectionAccessNoOne),
		MergeAccessLevel:       NormalizeProjectBranchProtectionAccessLevel(protection.MergeAccessLevel, ProjectBranchProtectionAccessMaintainer),
		RequireMergeRequest:    protection.RequireMergeRequest != 0,
		RequirePipelineSuccess: protection.RequirePipelineSuccess != 0,
		AllowForcePush:         protection.AllowForcePush != 0,
		AllowDelete:            protection.AllowDelete != 0,
	}
}

func NewProjectBranchUnprotectedEvent(projectID int64, branchName string) ProjectBranchProtectionChanged {
	return ProjectBranchProtectionChanged{
		Metadata:   domainevent.NewMetadata(),
		ProjectID:  projectID,
		BranchName: branchName,
		Protected:  false,
	}
}

type ProjectBranchDeleted struct {
	domainevent.Metadata
	ProjectID  int64  `json:"project_id"`
	BranchName string `json:"branch_name"`
}

func (ProjectBranchDeleted) Name() string {
	return EventProjectBranchDeleted
}

func NewProjectBranchDeletedEvent(projectID int64, branchName string) ProjectBranchDeleted {
	return ProjectBranchDeleted{
		Metadata:   domainevent.NewMetadata(),
		ProjectID:  projectID,
		BranchName: branchName,
	}
}
