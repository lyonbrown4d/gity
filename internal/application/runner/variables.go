package runner

import (
	"context"
	"regexp"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

var variableKeyPattern = regexp.MustCompile(`^[A-Z_][A-Z0-9_]*$`)

func (s *Service) ListProjectVariables(ctx context.Context, projectID int64) ([]VariableView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	if s.variableRepo == nil {
		return nil, oops.In("runner").With("project_id", projectID).New("project ci variable repository is not configured")
	}
	items, err := s.variableRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("runner").With("project_id", projectID).Wrapf(err, "list project ci variables")
	}
	return collectionlist.MapList(items, func(_ int, item cidomain.ProjectCIVariable) VariableView {
		return toVariableView(item)
	}).Values(), nil
}

func (s *Service) UpsertProjectVariable(ctx context.Context, projectID int64, input VariableInput) (VariableView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return VariableView{}, apperror.NotFound("project not found", err)
	}
	if s.variableRepo == nil {
		return VariableView{}, oops.In("runner").With("project_id", projectID).New("project ci variable repository is not configured")
	}
	normalized, err := normalizeVariableInput(projectID, input)
	if err != nil {
		return VariableView{}, err
	}
	item, err := s.variableRepo.Upsert(ctx, normalized)
	if err != nil {
		return VariableView{}, oops.In("runner").With("project_id", projectID, "key", input.Key).Wrapf(err, "upsert project ci variable")
	}
	return toVariableView(item), nil
}

func (s *Service) DeleteProjectVariable(ctx context.Context, projectID int64, key string) error {
	if s.variableRepo == nil {
		return oops.In("runner").With("project_id", projectID, "key", key).New("project ci variable repository is not configured")
	}
	if err := s.variableRepo.DeleteByProjectAndKey(ctx, projectID, key); err != nil {
		return oops.In("runner").With("project_id", projectID, "key", key).Wrapf(err, "delete project ci variable")
	}
	return nil
}

func normalizeVariableInput(projectID int64, input VariableInput) (gitports.UpsertProjectCIVariableInput, error) {
	key := strings.ToUpper(strings.TrimSpace(input.Key))
	if !variableKeyPattern.MatchString(key) {
		return gitports.UpsertProjectCIVariableInput{}, apperror.BadRequest("ci variable key is invalid", oops.In("runner").With("project_id", projectID, "key", input.Key).New("ci variable key is invalid"))
	}
	if input.Masked && len(input.Value) < 8 {
		return gitports.UpsertProjectCIVariableInput{}, apperror.BadRequest("masked ci variable value must be at least 8 characters", oops.In("runner").With("project_id", projectID, "key", key).New("masked ci variable value is too short"))
	}
	return gitports.UpsertProjectCIVariableInput{
		ProjectID: projectID,
		Key:       key,
		Value:     input.Value,
		Masked:    boolInt(input.Masked),
		Protected: boolInt(input.Protected),
	}, nil
}

func toVariableView(item cidomain.ProjectCIVariable) VariableView {
	value := item.Value
	if item.IsMasked() {
		value = ""
	}
	return VariableView{
		ID:        item.ID,
		ProjectID: item.ProjectID,
		Key:       item.Key,
		Value:     value,
		Masked:    item.IsMasked(),
		Protected: item.IsProtected(),
		CreatedAt: item.CreatedAt,
		UpdatedAt: item.UpdatedAt,
	}
}

func boolInt(value bool) int {
	if value {
		return 1
	}
	return 0
}
