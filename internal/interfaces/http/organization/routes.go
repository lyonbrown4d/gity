package organization

import (
	organizationservice "github.com/DaiYuANg/gity/internal/application/organization"
	organizationdomain "github.com/DaiYuANg/gity/internal/domain/organization"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/httpx"
	"strconv"
	"strings"
)

type createOrganizationInput struct {
	Body createOrganizationBody `json:"body"`
}

type organizationByIDInput struct {
	ID int64 `path:"id"`
}

type organizationsInput struct {
	IDs string `query:"ids"`
}

type organizationMemberInput struct {
	ID int64 `path:"id"`
}

type addOrganizationMemberInput struct {
	ID   int64                     `path:"id"`
	Body addOrganizationMemberBody `json:"body"`
}

type updateOrganizationInput struct {
	ID   int64                  `path:"id"`
	Body updateOrganizationBody `json:"body"`
}

type organizationOutput struct {
	Body any `json:"body"`
}

type createOrganizationBody struct {
	Key         string `json:"key"`
	Name        string `json:"name"`
	PathKey     string `json:"path_key"`
	OwnerUserID int64  `json:"owner_user_id"`
	Description string `json:"description"`
	Visibility  string `json:"visibility"`
}

type updateOrganizationBody struct {
	Key         *string `json:"key"`
	Name        *string `json:"name"`
	PathKey     *string `json:"path_key"`
	Description *string `json:"description"`
	Visibility  *string `json:"visibility"`
}

type addOrganizationMemberBody struct {
	UserID int64  `json:"user_id"`
	Role   string `json:"role"`
}

type organizationView struct {
	ID          string `json:"id"`
	Key         string `json:"key"`
	Name        string `json:"name"`
	Role        string `json:"role"`
	Description string `json:"description"`
	Visibility  string `json:"visibility"`
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
	service *organizationservice.Service
}

func NewEndpoint(service *organizationservice.Service) *Endpoint {
	return &Endpoint{service: service}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Organizations", "Organizations", "Organization APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/orgs", e.listOrganizations),
		httpapi.Get("/orgs/{id}", e.getOrganization),
		httpapi.Post("/orgs", e.createOrganization),
		httpapi.Patch("/orgs/{id}", e.updateOrganization),
		httpapi.Delete("/orgs/{id}", e.deleteOrganization),
		httpapi.Get("/orgs/{id}/members", e.listMembers),
		httpapi.Post("/orgs/{id}/members", e.addMember),
	)
}

func toOrganizationView(item organizationdomain.Organization) organizationView {
	return organizationView{
		ID:          strconv.FormatInt(item.ID, 10),
		Key:         item.PathKey,
		Name:        item.Name,
		Role:        "owner",
		Description: item.Description,
		Visibility:  item.Visibility,
	}
}

func toOrganizationMemberView(organizationID int64, item organizationservice.MemberView) organizationMemberView {
	return organizationMemberView{
		ID:             strconv.FormatInt(item.ID, 10),
		OrganizationID: strconv.FormatInt(organizationID, 10),
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
