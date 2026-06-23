package auth

import "github.com/arcgolabs/authx"

type Runtime struct {
	Engine *authx.Engine
}

func NewRuntime(engine *authx.Engine) *Runtime {
	return &Runtime{
		Engine: engine,
	}
}
