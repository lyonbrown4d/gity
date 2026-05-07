package user

import (
	"context"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
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
	service := e.service
	authRuntime := e.authRuntime

	listUsers := func(ctx context.Context, in *usersInput) (*userOutput, error) {
		items, err := service.List(ctx)
		if err != nil {
			return nil, err
		}
		views := make([]userView, 0, items.Len())
		idFilter := parseIDFilter(in.IDs)
		items.Range(func(_ int, item identity.User) bool {
			if len(idFilter) > 0 && !idFilter[item.ID] {
				return true
			}
			views = append(views, toUserView(item))
			return true
		})
		return &userOutput{Body: views}, nil
	}

	getCurrentUser := func(ctx context.Context, in *currentUserInput) (*userOutput, error) {
		user, err := currentUser(ctx, service, authRuntime, in.Authorization)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: toUserView(user)}, nil
	}

	getUser := func(ctx context.Context, in *userByIDInput) (*userOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: toUserView(item)}, nil
	}

	createUser := func(ctx context.Context, in *createUserInput) (*userOutput, error) {
		item, err := service.Create(ctx, userservice.CreateInput{
			Username:    in.Body.Username,
			DisplayName: in.Body.DisplayName,
			Email:       in.Body.Email,
		})
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: toUserView(item)}, nil
	}

	updateCurrentUser := func(ctx context.Context, in *updateCurrentUserInput) (*userOutput, error) {
		user, err := currentUser(ctx, service, authRuntime, in.Authorization)
		if err != nil {
			return nil, err
		}
		item, err := service.Update(ctx, user.ID, userservice.UpdateInput{
			Username:    in.Body.Username,
			DisplayName: in.Body.DisplayName,
			Email:       in.Body.Email,
			Status:      in.Body.Status,
		})
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: toUserView(item)}, nil
	}

	updateUser := func(ctx context.Context, in *updateUserInput) (*userOutput, error) {
		item, err := service.Update(ctx, in.ID, userservice.UpdateInput{
			Username:    in.Body.Username,
			DisplayName: in.Body.DisplayName,
			Email:       in.Body.Email,
			Status:      in.Body.Status,
		})
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: toUserView(item)}, nil
	}

	deleteUser := func(ctx context.Context, in *userByIDInput) (*userOutput, error) {
		item, err := service.GetByID(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		if err := service.Delete(ctx, in.ID); err != nil {
			return nil, err
		}
		return &userOutput{Body: toUserView(item)}, nil
	}

	listTokens := func(ctx context.Context, in *userTokenInput) (*userOutput, error) {
		items, err := service.ListTokens(ctx, in.ID)
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: items}, nil
	}

	createToken := func(ctx context.Context, in *createUserTokenInput) (*userOutput, error) {
		item, err := service.CreateToken(ctx, in.ID, userservice.CreateTokenInput{
			Name: in.Body.Name,
		})
		if err != nil {
			return nil, err
		}
		return &userOutput{Body: item}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/users", listUsers),
		httpapi.Get("/users/me", getCurrentUser),
		httpapi.Get("/users/{id}", getUser),
		httpapi.Post("/users", createUser),
		httpapi.Patch("/users/me", updateCurrentUser),
		httpapi.Patch("/users/{id}", updateUser),
		httpapi.Delete("/users/{id}", deleteUser),
		httpapi.Get("/users/{id}/tokens", listTokens),
		httpapi.Post("/users/{id}/tokens", createToken),
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

func parseIDFilter(raw string) map[int64]bool {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return nil
	}
	ids := map[int64]bool{}
	for _, part := range strings.Split(raw, ",") {
		id, err := strconv.ParseInt(strings.TrimSpace(part), 10, 64)
		if err == nil && id > 0 {
			ids[id] = true
		}
	}
	return ids
}
