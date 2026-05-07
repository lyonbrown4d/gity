package auth

import (
	"github.com/arcgolabs/authx"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
)

type Runtime struct {
	Engine *authx.Engine
}

func NewRuntime(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *namespacememberrepo.Repository) *Runtime {
	return &Runtime{Engine: newEngine(userRepository, tokenRepository, memberRepository)}
}
