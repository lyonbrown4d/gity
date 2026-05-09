package organization

import (
	"context"

	organizationservice "github.com/DaiYuANg/gity/internal/application/organization"
	organizationdomain "github.com/DaiYuANg/gity/internal/domain/organization"
	collectionlist "github.com/arcgolabs/collectionx/list"
)

func (e *Endpoint) listOrganizations(ctx context.Context, in *organizationsInput) (*organizationOutput, error) {
	items, err := e.service.List(ctx)
	if err != nil {
		return nil, err
	}
	idFilter := parseIDFilter(in.IDs)
	views := collectionlist.FilterMapList(items, func(_ int, item organizationdomain.Organization) (organizationView, bool) {
		if idFilter.Len() > 0 && !idFilter.Contains(item.ID) {
			return organizationView{}, false
		}
		return toOrganizationView(item), true
	}).Values()
	return &organizationOutput{Body: views}, nil
}

func (e *Endpoint) getOrganization(ctx context.Context, in *organizationByIDInput) (*organizationOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) createOrganization(ctx context.Context, in *createOrganizationInput) (*organizationOutput, error) {
	item, err := e.service.Create(ctx, buildCreateOrganizationInput(in))
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) updateOrganization(ctx context.Context, in *updateOrganizationInput) (*organizationOutput, error) {
	item, err := e.service.Update(ctx, in.ID, buildUpdateOrganizationInput(in))
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) deleteOrganization(ctx context.Context, in *organizationByIDInput) (*organizationOutput, error) {
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
	item, err := e.service.AddMember(ctx, in.ID, organizationservice.AddMemberInput{
		UserID: in.Body.UserID,
		Role:   in.Body.Role,
	})
	if err != nil {
		return nil, err
	}
	return &organizationOutput{Body: toOrganizationMemberView(in.ID, item)}, nil
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
