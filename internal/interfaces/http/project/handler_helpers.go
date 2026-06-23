package project

import (
	"strings"

	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	httpshared "github.com/lyonbrown4d/gity/internal/interfaces/http/shared"
)

func projectOrganizationFilter(in *projectsInput) *int64 {
	organizationID := in.OrganizationID
	if organizationID <= 0 {
		return nil
	}
	return &organizationID
}

func buildCreateProjectInput(in *createProjectInput) projectservice.CreateInput {
	organizationID := in.Body.OrganizationID
	return projectservice.CreateInput{
		OrganizationID: organizationID,
		Name:           in.Body.Name,
		PathKey:        httpshared.FirstNonEmpty(in.Body.PathKey, in.Body.Key),
		Visibility:     in.Body.Visibility,
		Description:    in.Body.Description,
		DefaultBranch:  in.Body.DefaultBranch,
	}
}

func repositoryRefName(primary, fallback string) string {
	refName := strings.TrimSpace(primary)
	if refName == "" {
		refName = strings.TrimSpace(fallback)
	}
	return refName
}

func buildCreateFileCommitInput(in *createFileCommitInput, branchName string) projectservice.CreateFileCommitInput {
	return projectservice.CreateFileCommitInput{
		BranchName:  branchName,
		Path:        in.Body.Path,
		Content:     in.Body.Content,
		Message:     in.Body.Message,
		AuthorName:  in.Body.AuthorName,
		AuthorEmail: in.Body.AuthorEmail,
	}
}
