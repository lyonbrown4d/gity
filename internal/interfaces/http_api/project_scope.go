package httpapi

import (
	"context"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/arcgolabs/httpx"
	"net/http"
)

type ProjectLookup interface {
	GetByID(context.Context, int64) (projectdomain.Project, error)
}

func ProjectScopeResolverFrom(lookup ProjectLookup) ProjectScopeResolver {
	return func(ctx context.Context, projectID int64) (infraauth.ProjectScope, error) {
		if lookup == nil {
			return infraauth.ProjectScope{}, httpx.NewError(http.StatusInternalServerError, "project lookup is not configured")
		}
		item, err := lookup.GetByID(ctx, projectID)
		if err != nil {
			return infraauth.ProjectScope{}, err
		}
		return infraauth.ProjectScope{ID: item.ID, OrganizationID: item.OrganizationID, Visibility: item.Visibility}, nil
	}
}
