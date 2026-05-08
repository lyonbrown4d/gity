package project

import (
	"strconv"
	"strings"

	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	"github.com/DaiYuANg/gity/internal/config"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	setx "github.com/arcgolabs/collectionx/set"
)

func toRepositoryView(item projectdomain.Project, settings config.Settings) repositoryView {
	baseURL := strings.TrimRight(settings.HTTP.BaseURL, "/")
	return repositoryView{
		ID:             strconv.FormatInt(item.ID, 10),
		UUID:           strconv.FormatInt(item.ID, 10),
		OrganizationID: strconv.FormatInt(item.NamespaceID, 10),
		Key:            item.PathKey,
		Name:           item.Name,
		Description:    item.Description,
		Visibility:     item.Visibility,
		DefaultBranch:  item.DefaultBranch,
		CloneHTTPURL:   baseURL + "/" + strings.Trim(item.FullPath, "/") + ".git",
	}
}

func toRepositoryBranchView(projectID int64, item projectservice.Branch) repositoryBranchView {
	return repositoryBranchView{
		RepositoryID:  strconv.FormatInt(projectID, 10),
		Name:          item.Name,
		IsProtected:   item.IsProtected,
		LastCommitSHA: item.LastCommitSHA,
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

func (in createProjectInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectByIDInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectByIDInput) ProjectIDValue() int64 {
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

func (in createFileCommitInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createFileCommitInput) ProjectIDValue() int64 {
	return in.ID
}

func parseIDFilter(raw string) *setx.Set[int64] {
	ids := setx.NewSet[int64]()
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ids
	}
	for part := range strings.SplitSeq(raw, ",") {
		id, err := strconv.ParseInt(strings.TrimSpace(part), 10, 64)
		if err == nil && id > 0 {
			ids.Add(id)
		}
	}
	return ids
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return value
		}
	}
	return ""
}
