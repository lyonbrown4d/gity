package organization

import (
	"context"
	"net/http"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/httpx"
	organizationservice "github.com/lyonbrown4d/gity/internal/application/organization"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
)

func (e *Endpoint) listOrganizations(ctx context.Context, in *organizationsInput) (*organizationOutput, error) {
	principal, err := e.requirePrincipal(ctx, in.Authorization)
	if err != nil {
		return nil, err
	}
	items, err := e.service.List(ctx)
	if err != nil {
		return nil, err
	}
	idFilter := parseIDFilter(in.IDs)
	organizationItems := items.Values()
	visible := make([]organizationdomain.Organization, 0, len(organizationItems))
	for i := range organizationItems {
		item := organizationItems[i]
		if idFilter.Len() > 0 && !idFilter.Contains(item.ID) {
			continue
		}
		allowed, err := e.canReadOrganization(ctx, principal, item)
		if err != nil {
			return nil, err
		}
		if allowed {
			visible = append(visible, item)
		}
	}
	views := collectionlist.MapList(collectionlist.NewList(visible...), func(_ int, item organizationdomain.Organization) organizationView {
		return toOrganizationView(item)
	}).Values()
	return &organizationOutput{Body: views}, nil
}

func (e *Endpoint) getOrganization(ctx context.Context, in *organizationByIDInput) (*organizationOutput, error) {
	item, err := e.requireOrganizationRead(ctx, in.Authorization, in.ID)
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) createOrganization(ctx context.Context, in *createOrganizationInput) (*organizationOutput, error) {
	principal, err := e.requirePrincipal(ctx, in.Authorization)
	if err != nil {
		return nil, err
	}
	input := buildCreateOrganizationInput(in)
	input.OwnerUserID = principal.UserID
	item, err := e.service.Create(ctx, input)
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) updateOrganization(ctx context.Context, in *updateOrganizationInput) (*organizationOutput, error) {
	if err := e.requireOrganizationManage(ctx, in.Authorization, in.ID); err != nil {
		return nil, err
	}
	item, err := e.service.Update(ctx, in.ID, buildUpdateOrganizationInput(in))
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) deleteOrganization(ctx context.Context, in *organizationByIDInput) (*organizationOutput, error) {
	if err := e.requireOrganizationManage(ctx, in.Authorization, in.ID); err != nil {
		return nil, err
	}
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	if err := e.service.Delete(ctx, in.ID); err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) listMembers(ctx context.Context, in *organizationMemberInput) (*organizationOutput, error) {
	if _, err := e.requireOrganizationRead(ctx, in.Authorization, in.ID); err != nil {
		return nil, err
	}
	items, err := e.service.ListMembers(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item organizationservice.MemberView) organizationMemberView {
		return toOrganizationMemberView(in.ID, item)
	}).Values()
	return &organizationOutput{Body: views}, nil
}

func (e *Endpoint) addMember(ctx context.Context, in *addOrganizationMemberInput) (*organizationOutput, error) {
	if err := e.requireOrganizationManage(ctx, in.Authorization, in.ID); err != nil {
		return nil, err
	}
	item, err := e.service.AddMember(ctx, in.ID, organizationservice.AddMemberInput{
		UserID: in.Body.UserID,
		Role:   in.Body.Role,
	})
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationMemberView(in.ID, item)}, nil
}

func (e *Endpoint) requirePrincipal(ctx context.Context, authorization string) (infraauth.Principal, error) {
	if e.authRuntime == nil {
		return infraauth.Principal{}, httpx.NewError(http.StatusInternalServerError, "auth runtime is not configured")
	}
	principal, ok, err := e.authRuntime.AuthenticateHeader(ctx, authorization)
	if err != nil {
		return infraauth.Principal{}, httpx.NewError(http.StatusUnauthorized, "invalid credentials", err)
	}
	if !ok {
		return infraauth.Principal{}, httpx.NewError(http.StatusUnauthorized, "authentication required")
	}
	return principal, nil
}

func (e *Endpoint) requireOrganizationRead(ctx context.Context, authorization string, organizationID int64) (organizationdomain.Organization, error) {
	principal, err := e.requirePrincipal(ctx, authorization)
	if err != nil {
		return organizationdomain.Organization{}, err
	}
	item, err := e.service.GetByID(ctx, organizationID)
	if err != nil {
		return organizationdomain.Organization{}, err
	}
	allowed, err := e.canReadOrganization(ctx, principal, item)
	if err != nil {
		return organizationdomain.Organization{}, err
	}
	if !allowed {
		return organizationdomain.Organization{}, httpx.NewError(http.StatusForbidden, "forbidden")
	}
	return item, nil
}

func (e *Endpoint) canReadOrganization(ctx context.Context, principal infraauth.Principal, item organizationdomain.Organization) (bool, error) {
	if principal.IsSuperAdmin {
		return true, nil
	}
	return e.service.CanRead(ctx, item, principal.UserID)
}

func (e *Endpoint) requireOrganizationManage(ctx context.Context, authorization string, organizationID int64) error {
	principal, err := e.requirePrincipal(ctx, authorization)
	if err != nil {
		return err
	}
	if principal.IsSuperAdmin {
		return nil
	}
	allowed, err := e.service.CanManage(ctx, organizationID, principal.UserID)
	if err != nil {
		return err
	}
	if !allowed {
		return httpx.NewError(http.StatusForbidden, "forbidden")
	}
	return nil
}

func buildCreateOrganizationInput(in *createOrganizationInput) organizationservice.CreateInput {
	return organizationservice.CreateInput{
		Name:        in.Body.Name,
		PathKey:     firstNonEmpty(in.Body.PathKey, in.Body.Key),
		OwnerUserID: in.Body.OwnerUserID,
		Description: in.Body.Description,
		Visibility:  in.Body.Visibility,
	}
}

func buildUpdateOrganizationInput(in *updateOrganizationInput) organizationservice.UpdateInput {
	pathKey := in.Body.PathKey
	if pathKey == nil {
		pathKey = in.Body.Key
	}
	return organizationservice.UpdateInput{
		Name:        in.Body.Name,
		PathKey:     pathKey,
		Description: in.Body.Description,
		Visibility:  in.Body.Visibility,
	}
}
