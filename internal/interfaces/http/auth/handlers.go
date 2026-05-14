package auth

import (
	"context"

	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
)

func (e *Endpoint) login(ctx context.Context, in *loginInput) (*authOutput, error) {
	session, err := e.service.Login(ctx, in.Body.Username)
	if err != nil {
		return nil, err
	}
	return &authOutput{Body: toAuthView(session)}, nil
}

func (e *Endpoint) refresh(ctx context.Context, in *refreshInput) (*authOutput, error) {
	session, err := e.service.Refresh(ctx, in.Body.RefreshToken)
	if err != nil {
		return nil, err
	}
	return &authOutput{Body: toAuthView(session)}, nil
}

func (e *Endpoint) logout(ctx context.Context, in *logoutInput) (*logoutOutput, error) {
	if token, ok := infraauth.TokenFromAuthorizationHeader(in.Authorization); ok {
		if err := e.service.RevokeToken(ctx, token); err != nil {
			return nil, err
		}
	}
	if err := e.service.RevokeToken(ctx, in.Body.RefreshToken); err != nil {
		return nil, err
	}
	out := &logoutOutput{}
	out.Body.OK = true
	return out, nil
}
