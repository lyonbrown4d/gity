package auth

import (
	organizationmemberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization_member"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	"github.com/arcgolabs/authx"
)

type Runtime struct {
	Engine *authx.Engine
}

func NewRuntime(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *organizationmemberrepo.Repository) *Runtime {
	return &Runtime{Engine: newEngine(userRepository, tokenRepository, memberRepository)}
}
