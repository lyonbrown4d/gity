package auth

import (
	"github.com/arcgolabs/authx"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
)

type Runtime struct {
	Engine *authx.Engine
}

func NewRuntime(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *organizationmemberrepo.Repository) *Runtime {
	return &Runtime{Engine: newEngine(userRepository, tokenRepository, memberRepository)}
}
