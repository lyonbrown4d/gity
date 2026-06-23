package project

import (
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	"github.com/lyonbrown4d/gity/internal/config"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_repo"
	"strconv"
	"strings"
	"time"
)

func toRepositoryView(item projectdomain.Project, settings config.Settings) repositoryView {
	baseURL := strings.TrimRight(settings.HTTP.BaseURL, "/")
	return repositoryView{
		ID:             strconv.FormatInt(item.ID, 10),
		UUID:           strconv.FormatInt(item.ID, 10),
		OrganizationID: strconv.FormatInt(item.OrganizationID, 10),
		Key:            item.PathKey,
		FullPath:       item.FullPath,
		Name:           item.Name,
		Description:    item.Description,
		Visibility:     item.Visibility,
		DefaultBranch:  item.DefaultBranch,
		Status:         item.Status,
		DeletedAt:      formatProjectTime(item.DeletedAt),
		CloneHTTPURL:   baseURL + "/" + strings.Trim(item.FullPath, "/") + ".git",
	}
}

func formatProjectTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format("2006-01-02T15:04:05Z")
}

func toRepositoryBranchView(projectID int64, item projectservice.Branch) repositoryBranchView {
	return repositoryBranchView{
		RepositoryID:  strconv.FormatInt(projectID, 10),
		Name:          item.Name,
		IsProtected:   item.IsProtected,
		LastCommitSHA: item.LastCommitSHA,
		Protection:    toBranchProtectionView(item.Protection),
	}
}

func toBranchProtectionView(item *projectservice.BranchProtection) *branchProtectionView {
	if item == nil {
		return nil
	}
	return &branchProtectionView{
		ID:                     strconv.FormatInt(item.ID, 10),
		RepositoryID:           strconv.FormatInt(item.ProjectID, 10),
		BranchName:             item.BranchName,
		RuleType:               item.RuleType,
		PushAccessLevel:        item.PushAccessLevel,
		MergeAccessLevel:       item.MergeAccessLevel,
		RequireMergeRequest:    item.RequireMergeRequest,
		RequirePipelineSuccess: item.RequirePipelineSuccess,
		AllowForcePush:         item.AllowForcePush,
		AllowDelete:            item.AllowDelete,
	}
}

func toRepositoryCommitView(projectID int64, branchName string, item gitrepo.Commit) repositoryCommitView {
	return repositoryCommitView{
		RepositoryID: strconv.FormatInt(projectID, 10),
		BranchName:   branchName,
		CommitSHA:    item.Hash,
		Message:      item.Message,
		AuthorUserID: item.AuthorName,
		CreatedAt:    item.CommittedAt,
	}
}

func toRepositoryTreeEntryView(item gitrepo.TreeEntry) repositoryTreeEntryView {
	return repositoryTreeEntryView{
		Name: item.Name,
		Path: item.Path,
		Kind: item.Type,
		OID:  item.Mode,
		Size: item.Size,
	}
}

func toRepositoryBlobView(item gitrepo.Blob) repositoryBlobView {
	return repositoryBlobView{
		Path:     item.Path,
		Content:  item.Content,
		Size:     item.Size,
		IsBinary: item.Encoding == "base64",
		Encoding: item.Encoding,
	}
}

func (in projectsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createProjectInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectByIDInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectByIDInput) ProjectIDValue() int64 {
	return in.ID
}

func (in projectMemberInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectMemberInput) ProjectIDValue() int64 {
	return in.ID
}

func (in createProjectMemberInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createProjectMemberInput) ProjectIDValue() int64 {
	return in.ID
}

func (in upsertProjectMemberInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in upsertProjectMemberInput) ProjectIDValue() int64 {
	return in.ID
}

func (in deleteProjectInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in deleteProjectInput) ProjectIDValue() int64 {
	return in.ID
}

func (in createBranchInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createBranchInput) ProjectIDValue() int64 {
	return in.ID
}

func (in branchProtectionInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in branchProtectionInput) ProjectIDValue() int64 {
	return in.ID
}

func (in upsertBranchProtectionInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in upsertBranchProtectionInput) ProjectIDValue() int64 {
	return in.ID
}

func (in createFileCommitInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createFileCommitInput) ProjectIDValue() int64 {
	return in.ID
}

func (in projectRepositoryInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectRepositoryInput) ProjectIDValue() int64 {
	return in.ID
}

func (in projectRepositorySearchInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectRepositorySearchInput) ProjectIDValue() int64 {
	return in.ID
}

func (in refreshProjectSearchIndexInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in refreshProjectSearchIndexInput) ProjectIDValue() int64 {
	return in.ID
}
