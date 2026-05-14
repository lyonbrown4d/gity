package auth

import (
	"github.com/arcgolabs/authx"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
	projectmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_member"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
)

type Runtime struct {
	Engine           *authx.Engine
	memberRepository *organizationmemberrepo.Repository
	projectMembers   *projectmemberrepo.Repository
}

func NewRuntime(userRepository *userrepo.Repository, tokenRepository *usertokenrepo.Repository, memberRepository *organizationmemberrepo.Repository, projectMembers *projectmemberrepo.Repository) *Runtime {
	return &Runtime{
		Engine:           newEngine(userRepository, tokenRepository, memberRepository, projectMembers),
		memberRepository: memberRepository,
		projectMembers:   projectMembers,
	}
}
