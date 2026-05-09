package namespace

import (
	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
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
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/namespaces", e.listNamespaces),
		httpapi.Get("/orgs", e.listNamespaces),
		httpapi.Get("/namespaces/{id}", e.getNamespace),
		httpapi.Get("/orgs/{id}", e.getNamespace),
		httpapi.Post("/namespaces", e.createNamespace),
		httpapi.Post("/orgs", e.createNamespace),
		httpapi.Patch("/namespaces/{id}", e.updateNamespace),
		httpapi.Patch("/orgs/{id}", e.updateNamespace),
		httpapi.Delete("/namespaces/{id}", e.deleteNamespace),
		httpapi.Delete("/orgs/{id}", e.deleteNamespace),
		httpapi.Get("/namespaces/{id}/members", e.listMembers),
		httpapi.Get("/orgs/{id}/members", e.listMembers),
		httpapi.Post("/namespaces/{id}/members", e.addMember),
		httpapi.Post("/orgs/{id}/members", e.addMember),
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
