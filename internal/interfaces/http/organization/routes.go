package organization

import (
	"github.com/arcgolabs/httpx"
	organizationservice "github.com/lyonbrown4d/gity/internal/application/organization"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
	"strconv"
)

type createOrganizationInput struct {
	Authorization string                 `header:"Authorization"`
	Body          createOrganizationBody `json:"body"`
}

type organizationByIDInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type organizationsInput struct {
	Authorization string `header:"Authorization"`
	IDs           string `query:"ids"`
}

type organizationMemberInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type addOrganizationMemberInput struct {
	ID            int64                     `path:"id"`
	Authorization string                    `header:"Authorization"`
	Body          addOrganizationMemberBody `json:"body"`
}

type updateOrganizationInput struct {
	ID            int64                  `path:"id"`
	Authorization string                 `header:"Authorization"`
	Body          updateOrganizationBody `json:"body"`
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
	service     *organizationservice.Service
	authRuntime *infraauth.Runtime
}

func NewEndpoint(service *organizationservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Organizations", "Organizations", "Organization APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/orgs", e.listOrganizations, httpapi.RequireUserRoute[organizationsInput, organizationOutput](e.authRuntime)),
		httpapi.Get("/orgs/{id}", e.getOrganization, httpapi.RequireUserRoute[organizationByIDInput, organizationOutput](e.authRuntime)),
		httpapi.Post("/orgs", e.createOrganization, httpapi.RequireUserRoute[createOrganizationInput, organizationOutput](e.authRuntime)),
		httpapi.Patch("/orgs/{id}", e.updateOrganization, httpapi.RequireUserRoute[updateOrganizationInput, organizationOutput](e.authRuntime)),
		httpapi.Delete("/orgs/{id}", e.deleteOrganization, httpapi.RequireUserRoute[organizationByIDInput, organizationOutput](e.authRuntime)),
		httpapi.Get("/orgs/{id}/members", e.listMembers, httpapi.RequireUserRoute[organizationMemberInput, organizationOutput](e.authRuntime)),
		httpapi.Post("/orgs/{id}/members", e.addMember, httpapi.RequireUserRoute[addOrganizationMemberInput, organizationOutput](e.authRuntime)),
	)
}

func (in createOrganizationInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in organizationByIDInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in organizationsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in organizationMemberInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in addOrganizationMemberInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateOrganizationInput) AuthorizationHeader() string {
	return in.Authorization
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
