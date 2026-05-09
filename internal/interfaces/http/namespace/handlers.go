package namespace

import (
	"context"

	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	collectionlist "github.com/arcgolabs/collectionx/list"
)

func (e *Endpoint) listNamespaces(ctx context.Context, in *namespacesInput) (*namespaceOutput, error) {
	items, err := e.service.List(ctx)
	if err != nil {
		return nil, err
	}
	idFilter := parseIDFilter(in.IDs)
	views := collectionlist.FilterMapList(items, func(_ int, item namespacedomain.Namespace) (organizationView, bool) {
		if idFilter.Len() > 0 && !idFilter.Contains(item.ID) {
			return organizationView{}, false
		}
		return toOrganizationView(item), true
	}).Values()
	return &namespaceOutput{Body: views}, nil
}

func (e *Endpoint) getNamespace(ctx context.Context, in *namespaceByIDInput) (*namespaceOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	return &namespaceOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) createNamespace(ctx context.Context, in *createNamespaceInput) (*namespaceOutput, error) {
	item, err := e.service.Create(ctx, buildCreateNamespaceInput(in))
	if err != nil {
		return nil, err
	}
	return &namespaceOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) updateNamespace(ctx context.Context, in *updateNamespaceInput) (*namespaceOutput, error) {
	item, err := e.service.Update(ctx, in.ID, buildUpdateNamespaceInput(in))
	if err != nil {
		return nil, err
	}
	return &namespaceOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) deleteNamespace(ctx context.Context, in *namespaceByIDInput) (*namespaceOutput, error) {
	item, err := e.service.GetByID(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	if err := e.service.Delete(ctx, in.ID); err != nil {
		return nil, err
	}
	return &namespaceOutput{Body: toOrganizationView(item)}, nil
}

func (e *Endpoint) listMembers(ctx context.Context, in *namespaceMemberInput) (*namespaceOutput, error) {
	items, err := e.service.ListMembers(ctx, in.ID)
	if err != nil {
		return nil, err
	}
	views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item namespaceservice.MemberView) organizationMemberView {
		return toOrganizationMemberView(in.ID, item)
	}).Values()
	return &namespaceOutput{Body: views}, nil
}

func (e *Endpoint) addMember(ctx context.Context, in *addNamespaceMemberInput) (*namespaceOutput, error) {
	item, err := e.service.AddMember(ctx, in.ID, namespaceservice.AddMemberInput{
		UserID: in.Body.UserID,
		Role:   in.Body.Role,
	})
	if err != nil {
		return nil, err
	}
	return &namespaceOutput{Body: toOrganizationMemberView(in.ID, item)}, nil
}

func buildCreateNamespaceInput(in *createNamespaceInput) namespaceservice.CreateInput {
	return namespaceservice.CreateInput{
		Kind:        in.Body.Kind,
		Name:        in.Body.Name,
		PathKey:     firstNonEmpty(in.Body.PathKey, in.Body.Key),
		OwnerUserID: in.Body.OwnerUserID,
		Description: in.Body.Description,
	}
}

func buildUpdateNamespaceInput(in *updateNamespaceInput) namespaceservice.UpdateInput {
	pathKey := in.Body.PathKey
	if pathKey == nil {
		pathKey = in.Body.Key
	}
	return namespaceservice.UpdateInput{
		Kind:        in.Body.Kind,
		Name:        in.Body.Name,
		PathKey:     pathKey,
		Description: in.Body.Description,
	}
}
