package httpapi

import (
	"context"
	"net/http"

	"github.com/DaiYuANg/gity/internal/entity"
	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	"github.com/arcgolabs/httpx"
)

type ProjectLookup interface {
	GetByID(context.Context, int64) (entity.Project, error)
}

func ProjectScopeResolverFrom(lookup ProjectLookup) ProjectScopeResolver {
	return func(ctx context.Context, projectID int64) (platformauth.ProjectScope, error) {
		if lookup == nil {
			return platformauth.ProjectScope{}, httpx.NewError(http.StatusInternalServerError, "project lookup is not configured")
		}
		item, err := lookup.GetByID(ctx, projectID)
		if err != nil {
			return platformauth.ProjectScope{}, err
		}
		return platformauth.ProjectScope{ID: item.ID, NamespaceID: item.NamespaceID, Visibility: item.Visibility}, nil
	}
}
