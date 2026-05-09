package auth

import (
	"strconv"

	userservice "github.com/DaiYuANg/gity/internal/application/user"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
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
	httpapi.MustRegisterRoutes(registrar,
		httpapi.Post("/auth/login", e.login),
		httpapi.Post("/auth/refresh", e.refresh),
		httpapi.Post("/auth/logout", e.logout),
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
