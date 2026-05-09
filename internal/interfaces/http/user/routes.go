package user

import (
	"context"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	setx "github.com/arcgolabs/collectionx/set"
	"strconv"
	"strings"

	userservice "github.com/DaiYuANg/gity/internal/application/user"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/arcgolabs/httpx"
)

type createUserInput struct {
	Body createUserBody `json:"body"`
}

type userByIDInput struct {
	ID int64 `path:"id"`
}

type currentUserInput struct {
	Authorization string `header:"Authorization"`
}

type userTokenInput struct {
	ID int64 `path:"id"`
}

type createUserTokenInput struct {
	ID   int64               `path:"id"`
	Body createUserTokenBody `json:"body"`
}

type updateUserInput struct {
	ID   int64          `path:"id"`
	Body updateUserBody `json:"body"`
}

type updateCurrentUserInput struct {
	Authorization string         `header:"Authorization"`
	Body          updateUserBody `json:"body"`
}

type userOutput struct {
	Body any `json:"body"`
}

type usersInput struct {
	IDs string `query:"ids"`
}

type createUserBody struct {
	Username    string `json:"username"`
	DisplayName string `json:"display_name"`
	Email       string `json:"email"`
	Password    string `json:"password"`
}

type updateUserBody struct {
	Username    *string `json:"username"`
	DisplayName *string `json:"display_name"`
	Email       *string `json:"email"`
	Status      *string `json:"status"`
	Password    *string `json:"password"`
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
		httpapi.Get("/users", e.listUsers),
		httpapi.Get("/users/me", e.getCurrentUser),
		httpapi.Get("/users/{id}", e.getUser),
		httpapi.Post("/users", e.createUser),
		httpapi.Patch("/users/me", e.updateCurrentUser),
		httpapi.Patch("/users/{id}", e.updateUser),
		httpapi.Delete("/users/{id}", e.deleteUser),
		httpapi.Get("/users/{id}/tokens", e.listTokens),
		httpapi.Post("/users/{id}/tokens", e.createToken),
	)
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
		IsSuperAdmin: false,
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
