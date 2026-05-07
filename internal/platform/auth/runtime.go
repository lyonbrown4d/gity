package auth

import (
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespacemember"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/usertoken"
	"github.com/arcgolabs/authx"
)

type Runtime struct {
	Engine *authx.Engine
}

func NewRuntime(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *namespacememberrepo.Repository) *Runtime {
	return &Runtime{Engine: newEngine(userRepository, tokenRepository, memberRepository)}
}
