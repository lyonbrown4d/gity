package auth

import (
	"context"
	"strconv"

	userservice "github.com/DaiYuANg/gity/internal/application/user"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/interfaces/httpapi"
	"github.com/arcgolabs/httpx"
)

type loginInput struct {
	Body loginBody `json:"body"`
}

type refreshInput struct {
	Body refreshBody `json:"body"`
}

type logoutInput struct {
	Authorization string     `header:"Authorization"`
	Body          logoutBody `json:"body"`
}

type loginBody struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type refreshBody struct {
	RefreshToken string `json:"refresh_token"`
}

type logoutBody struct {
	RefreshToken string `json:"refresh_token"`
}

type authOutput struct {
	Body authView `json:"body"`
}

type logoutOutput struct {
	Body struct {
		OK bool `json:"ok"`
	} `json:"body"`
}

type authView struct {
	UserID       string `json:"user_id"`
	Username     string `json:"username"`
	Token        string `json:"token"`
	RefreshToken string `json:"refresh_token"`
}

type Endpoint struct {
	service *userservice.Service
}

func NewEndpoint(service *userservice.Service) *Endpoint {
	return &Endpoint{service: service}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Auth", "Auth", "Authentication APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service

	login := func(ctx context.Context, in *loginInput) (*authOutput, error) {
		session, err := service.Login(ctx, in.Body.Username)
		if err != nil {
			return nil, err
		}
		return &authOutput{Body: toAuthView(session)}, nil
	}

	refresh := func(ctx context.Context, in *refreshInput) (*authOutput, error) {
		session, err := service.Refresh(ctx, in.Body.RefreshToken)
		if err != nil {
			return nil, err
		}
		return &authOutput{Body: toAuthView(session)}, nil
	}

	logout := func(ctx context.Context, in *logoutInput) (*logoutOutput, error) {
		if token, ok := infraauth.TokenFromAuthorizationHeader(in.Authorization); ok {
			if err := service.RevokeToken(ctx, token); err != nil {
				return nil, err
			}
		}
		if err := service.RevokeToken(ctx, in.Body.RefreshToken); err != nil {
			return nil, err
		}
		out := &logoutOutput{}
		out.Body.OK = true
		return out, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Post("/auth/login", login),
		httpapi.Post("/auth/refresh", refresh),
		httpapi.Post("/auth/logout", logout),
	)
}

func toAuthView(session userservice.AuthSession) authView {
	return authView{
		UserID:       strconv.FormatInt(session.User.ID, 10),
		Username:     session.User.Username,
		Token:        session.AccessToken.Token,
		RefreshToken: session.RefreshToken.Token,
	}
}
