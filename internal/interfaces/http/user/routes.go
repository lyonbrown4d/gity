package user

import (
	"context"
	"strconv"
	"strings"

	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/httpx"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type createUserInput struct {
	Authorization string         `header:"Authorization"`
	Body          createUserBody `json:"body"`
}

type userByIDInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type currentUserInput struct {
	Authorization string `header:"Authorization"`
}

type userTokenInput struct {
	ID            int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type createUserTokenInput struct {
	ID            int64               `path:"id"`
	Authorization string              `header:"Authorization"`
	Body          createUserTokenBody `json:"body"`
}

type updateUserInput struct {
	ID            int64          `path:"id"`
	Authorization string         `header:"Authorization"`
	Body          updateUserBody `json:"body"`
}

type updateCurrentUserInput struct {
	Authorization string         `header:"Authorization"`
	Body          updateUserBody `json:"body"`
}

type userOutput struct {
	Body any `json:"body"`
}

type usersInput struct {
	Authorization string `header:"Authorization"`
	IDs           string `query:"ids"`
}

type createUserBody struct {
	Username     string `json:"username"`
	DisplayName  string `json:"display_name"`
	Email        string `json:"email"`
	Password     string `json:"password"`
	IsSuperAdmin bool   `json:"is_super_admin"`
}

type updateUserBody struct {
	Username     *string `json:"username"`
	DisplayName  *string `json:"display_name"`
	Email        *string `json:"email"`
	Status       *string `json:"status"`
	Password     *string `json:"password"`
	IsSuperAdmin *bool   `json:"is_super_admin"`
}

type createUserTokenBody struct {
	Name string `json:"name"`
}

type userView struct {
	ID           string `json:"id"`
	Username     string `json:"username"`
	DisplayName  string `json:"display_name"`
	Email        string `json:"email"`
	Status       string `json:"status"`
	IsSuperAdmin bool   `json:"is_super_admin"`
}

type Endpoint struct {
	service     *userservice.Service
	authRuntime *infraauth.Runtime
}

func NewEndpoint(service *userservice.Service, authRuntime *infraauth.Runtime) *Endpoint {
	return &Endpoint{service: service, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Users", "Users", "User APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/users", e.listUsers, httpapi.RequireUserRoute[usersInput, userOutput](e.authRuntime)),
		httpapi.Get("/users/me", e.getCurrentUser, httpapi.RequireUserRoute[currentUserInput, userOutput](e.authRuntime)),
		httpapi.Get("/users/{id}", e.getUser, httpapi.RequireUserRoute[userByIDInput, userOutput](e.authRuntime)),
		httpapi.Post("/users", e.createUser, httpapi.RequireUserRoute[createUserInput, userOutput](e.authRuntime)),
		httpapi.Patch("/users/me", e.updateCurrentUser, httpapi.RequireUserRoute[updateCurrentUserInput, userOutput](e.authRuntime)),
		httpapi.Patch("/users/{id}", e.updateUser, httpapi.RequireUserRoute[updateUserInput, userOutput](e.authRuntime)),
		httpapi.Delete("/users/{id}", e.deleteUser, httpapi.RequireUserRoute[userByIDInput, userOutput](e.authRuntime)),
		httpapi.Get("/users/{id}/tokens", e.listTokens, httpapi.RequireUserRoute[userTokenInput, userOutput](e.authRuntime)),
		httpapi.Post("/users/{id}/tokens", e.createToken, httpapi.RequireUserRoute[createUserTokenInput, userOutput](e.authRuntime)),
	)
}

func (in createUserInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in userByIDInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in currentUserInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in userTokenInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createUserTokenInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateUserInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateCurrentUserInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in usersInput) AuthorizationHeader() string {
	return in.Authorization
}

func currentUser(ctx context.Context, service *userservice.Service, authRuntime *infraauth.Runtime, authorization string) (identity.User, error) {
	principal, ok, err := authRuntime.AuthenticateHeader(ctx, authorization)
	if err != nil {
		return identity.User{}, err
	}
	if !ok {
		return identity.User{}, httpx.NewError(401, "authentication required", nil)
	}
	return service.GetByID(ctx, principal.UserID)
}

func toUserView(item identity.User) userView {
	return userView{
		ID:           strconv.FormatInt(item.ID, 10),
		Username:     item.Username,
		DisplayName:  item.DisplayName,
		Email:        item.Email,
		Status:       "active",
		IsSuperAdmin: item.IsSuperAdmin != 0,
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
