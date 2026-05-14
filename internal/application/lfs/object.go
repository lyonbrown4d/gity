package lfs

import (
	"context"
	"strconv"

	collectionlist "github.com/arcgolabs/collectionx/list"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	lfsdomain "github.com/lyonbrown4d/gity/internal/domain/lfs"
	"github.com/samber/oops"
)

const defaultObjectPageSize = 100

type ObjectListInput struct {
	Cursor string
	Limit  int
}

type ObjectView struct {
	ID        string `json:"id"`
	ProjectID string `json:"project_id"`
	OID       string `json:"oid"`
	ByteSize  int64  `json:"byte_size"`
	CreatedAt string `json:"created_at,omitempty"`
	UpdatedAt string `json:"updated_at,omitempty"`
}

type ObjectListResult struct {
	Objects    []ObjectView `json:"objects"`
	NextCursor string       `json:"next_cursor,omitempty"`
}

func (s *Service) ListObjects(ctx context.Context, projectID int64, input ObjectListInput) (ObjectListResult, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return ObjectListResult{}, apperror.NotFound("project not found", err)
	}
	afterID, err := parseCursor(input.Cursor)
	if err != nil {
		return ObjectListResult{}, apperror.BadRequest("invalid lfs object cursor", err)
	}
	limit := normalizeObjectLimit(input.Limit)
	items, err := s.objectRepo.ListByProjectID(ctx, storageports.ListProjectLFSObjectsInput{ProjectID: projectID, AfterID: afterID, Limit: limit + 1})
	if err != nil {
		return ObjectListResult{}, oops.In("lfs").With("project_id", projectID, "after_id", afterID, "limit", limit).Wrapf(err, "list lfs objects")
	}
	values := items.Values()
	nextCursor := ""
	if len(values) > limit {
		nextCursor = strconv.FormatInt(values[limit-1].ID, 10)
		values = values[:limit]
	}
	views := collectionlist.MapList(collectionlist.NewList(values...), func(_ int, item lfsdomain.ProjectLFSObject) ObjectView {
		return buildObjectView(item)
	}).Values()
	return ObjectListResult{Objects: views, NextCursor: nextCursor}, nil
}

func buildObjectView(item lfsdomain.ProjectLFSObject) ObjectView {
	return ObjectView{
		ID:        strconv.FormatInt(item.ID, 10),
		ProjectID: strconv.FormatInt(item.ProjectID, 10),
		OID:       item.OID,
		ByteSize:  item.ByteSize,
		CreatedAt: item.CreatedAt.UTC().Format(timeLayoutRFC3339Millis),
		UpdatedAt: item.UpdatedAt.UTC().Format(timeLayoutRFC3339Millis),
	}
}

func normalizeObjectLimit(limit int) int {
	if limit <= 0 || limit > defaultObjectPageSize {
		return defaultObjectPageSize
	}
	return limit
}
