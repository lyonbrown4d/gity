package auth

import "github.com/DaiYuANg/arcgo/authx"

type Runtime struct {
	Engine *authx.Engine
}

func NewRuntime() *Runtime {
	return &Runtime{Engine: authx.NewEngine()}
}
