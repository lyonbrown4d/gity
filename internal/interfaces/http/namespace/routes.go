package namespace

import (
	"context"
	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	collectionlist "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/httpx"
	"strconv"
	"strings"
)

type createNamespaceInput struct {
	Body createNamespaceBody `json:"body"`
}

type namespaceByIDInput struct {
	ID int64 `path:"id"`
}

type namespacesInput struct {
	IDs string `query:"ids"`
}

type namespaceMemberInput struct {
	ID int64 `path:"id"`
}

type addNamespaceMemberInput struct {
	ID   int64                  `path:"id"`
	Body addNamespaceMemberBody `json:"body"`
}

type updateNamespaceInput struct {
	ID   int64               `path:"id"`
	Body updateNamespaceBody `json:"body"`
}

type namespaceOutput struct {
	Body any `json:"body"`
}

type createNamespaceBody struct {
	Kind        string `json:"kind"`
	Key         string `json:"key"`
	Name        string `json:"name"`
	PathKey     string `json:"path_key"`
	OwnerUserID int64  `json:"owner_user_id"`
	Description string `json:"description"`
}

type updateNamespaceBody struct {
	Kind        *string `json:"kind"`
	Key         *string `json:"key"`
	Name        *string `json:"name"`
	PathKey     *string `json:"path_key"`
	Description *string `json:"description"`
}

type addNamespaceMemberBody struct {
	UserID int64  `json:"user_id"`
	Role   string `json:"role"`
}

type organizationView struct {
	ID          string `json:"id"`
	Key         string `json:"key"`
	Name        string `json:"name"`
	Role        string `json:"role"`
	Description string `json:"description"`
}

type organizationMemberView struct {
	ID             string `json:"id"`
	OrganizationID string `json:"organization_id"`
	UserID         string `json:"user_id"`
	Username       string `json:"username"`
	Email          string `json:"email"`
	Role           string `json:"role"`
}

type Endpoint struct {
	service *namespaceservice.Service
}

func NewEndpoint(service *namespaceservice.Service) *Endpoint {
	return &Endpoint{service: service}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Organizations", "Organizations", "Organization and namespace APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service

	listNamespaces := func(ctx context.Context, in *namespacesInput) (*namespaceOutput, error) {
		items, err := service.List(ctx)
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

	getNamespace := func(ctx context.Context, in *namespaceByIDInput) (*namespaceOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: toOrganizationView(item)}, nil
	}

	createNamespace := func(ctx context.Context, in *createNamespaceInput) (*namespaceOutput, error) {
		pathKey := firstNonEmpty(in.Body.PathKey, in.Body.Key)
		item, err := service.Create(ctx, namespaceservice.CreateInput{
			Kind:        in.Body.Kind,
			Name:        in.Body.Name,
			PathKey:     pathKey,
			OwnerUserID: in.Body.OwnerUserID,
			Description: in.Body.Description,
		})
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: toOrganizationView(item)}, nil
	}

	updateNamespace := func(ctx context.Context, in *updateNamespaceInput) (*namespaceOutput, error) {
		pathKey := in.Body.PathKey
		if pathKey == nil {
			pathKey = in.Body.Key
		}
		item, err := service.Update(ctx, in.ID, namespaceservice.UpdateInput{
			Kind:        in.Body.Kind,
			Name:        in.Body.Name,
			PathKey:     pathKey,
			Description: in.Body.Description,
		})
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: toOrganizationView(item)}, nil
	}

	deleteNamespace := func(ctx context.Context, in *namespaceByIDInput) (*namespaceOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		if err := service.Delete(ctx, in.ID); err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: toOrganizationView(item)}, nil
	}

	listMembers := func(ctx context.Context, in *namespaceMemberInput) (*namespaceOutput, error) {
		items, err := service.ListMembers(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		views := collectionlist.MapList(collectionlist.NewList(items...), func(_ int, item namespaceservice.MemberView) organizationMemberView {
			return toOrganizationMemberView(in.ID, item)
		}).Values()
		return &namespaceOutput{Body: views}, nil
	}

	addMember := func(ctx context.Context, in *addNamespaceMemberInput) (*namespaceOutput, error) {
		item, err := service.AddMember(ctx, in.ID, namespaceservice.AddMemberInput{
			UserID: in.Body.UserID,
			Role:   in.Body.Role,
		})
		if err != nil {
			return nil, err
		}
		return &namespaceOutput{Body: toOrganizationMemberView(in.ID, item)}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/namespaces", listNamespaces),
		httpapi.Get("/orgs", listNamespaces),
		httpapi.Get("/namespaces/{id}", getNamespace),
		httpapi.Get("/orgs/{id}", getNamespace),
		httpapi.Post("/namespaces", createNamespace),
		httpapi.Post("/orgs", createNamespace),
		httpapi.Patch("/namespaces/{id}", updateNamespace),
		httpapi.Patch("/orgs/{id}", updateNamespace),
		httpapi.Delete("/namespaces/{id}", deleteNamespace),
		httpapi.Delete("/orgs/{id}", deleteNamespace),
		httpapi.Get("/namespaces/{id}/members", listMembers),
		httpapi.Get("/orgs/{id}/members", listMembers),
		httpapi.Post("/namespaces/{id}/members", addMember),
		httpapi.Post("/orgs/{id}/members", addMember),
	)
}

func toOrganizationView(item namespacedomain.Namespace) organizationView {
	return organizationView{
		ID:          strconv.FormatInt(item.ID, 10),
		Key:         item.PathKey,
		Name:        item.Name,
		Role:        "owner",
		Description: item.Description,
	}
}

func toOrganizationMemberView(namespaceID int64, item namespaceservice.MemberView) organizationMemberView {
	return organizationMemberView{
		ID:             strconv.FormatInt(item.ID, 10),
		OrganizationID: strconv.FormatInt(namespaceID, 10),
		UserID:         strconv.FormatInt(item.UserID, 10),
		Username:       item.Username,
		Email:          item.Email,
		Role:           item.Role,
	}
}

func parseIDFilter(raw string) *setx.Set[int64] {
	ids := setx.NewSet[int64]()
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ids
	}
	for _, part := range strings.Split(raw, ",") {
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
