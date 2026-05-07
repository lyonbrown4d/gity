package httpapi

import (
	"github.com/DaiYuANg/gity/internal/infrastructure/auth"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/httpx"
)

type EndpointRoute interface {
	register(group *httpx.Group)
}

type RouteOption[I, O any] func(*routeConfig[I, O])

type routeConfig[I, O any] struct {
	operations []httpx.OperationOption
	policies   []httpx.RoutePolicy[I, O]
}

type route[I, O any] struct {
	method  string
	path    string
	handler httpx.TypedHandler[I, O]
	config  routeConfig[I, O]
}

func Get[I, O any](path string, handler httpx.TypedHandler[I, O], options ...RouteOption[I, O]) EndpointRoute {
	return Route(httpx.MethodGet, path, handler, options...)
}

func Post[I, O any](path string, handler httpx.TypedHandler[I, O], options ...RouteOption[I, O]) EndpointRoute {
	return Route(httpx.MethodPost, path, handler, options...)
}

func Patch[I, O any](path string, handler httpx.TypedHandler[I, O], options ...RouteOption[I, O]) EndpointRoute {
	return Route(httpx.MethodPatch, path, handler, options...)
}

func Delete[I, O any](path string, handler httpx.TypedHandler[I, O], options ...RouteOption[I, O]) EndpointRoute {
	return Route(httpx.MethodDelete, path, handler, options...)
}

func Route[I, O any](method string, path string, handler httpx.TypedHandler[I, O], options ...RouteOption[I, O]) EndpointRoute {
	cfg := routeConfig[I, O]{}
	for _, option := range options {
		if option != nil {
			option(&cfg)
		}
	}
	return route[I, O]{
		method:  method,
		path:    path,
		handler: handler,
		config:  cfg,
	}
}

func Operation[I, O any](option httpx.OperationOption) RouteOption[I, O] {
	return func(config *routeConfig[I, O]) {
		if option != nil {
			config.operations = append(config.operations, option)
		}
	}
}

func DeprecatedRoute[I, O any](reason string) RouteOption[I, O] {
	return Operation[I, O](Deprecated(reason))
}

func Policy[I, O any](policy httpx.RoutePolicy[I, O]) RouteOption[I, O] {
	return func(config *routeConfig[I, O]) {
		config.policies = append(config.policies, policy)
	}
}

func RequireUserRoute[I AuthorizationInput, O any](authRuntime *auth.Runtime) RouteOption[I, O] {
	return Policy(RequireUser[I, O](authRuntime))
}

func RequireProjectWriteRoute[I ProjectInput, O any](authRuntime *auth.Runtime, resolver ProjectScopeResolver) RouteOption[I, O] {
	return Policy(RequireProjectWrite[I, O](authRuntime, resolver))
}

func MustRegisterRoutes(registrar httpx.Registrar, routes ...EndpointRoute) {
	group := registrar.Scope()
	for _, route := range routes {
		if route != nil {
			route.register(group)
		}
	}
}

func RegisterEndpoints(server httpx.ServerRuntime, endpoints *collectionlist.List[httpx.Endpoint]) {
	if server == nil || endpoints == nil {
		return
	}
	endpoints.Range(func(_ int, endpoint httpx.Endpoint) bool {
		server.RegisterOnly(endpoint)
		return true
	})
}

func (r route[I, O]) register(group *httpx.Group) {
	if len(r.config.policies) == 0 {
		httpx.MustGroupRoute(group, r.method, r.path, r.handler, r.config.operations...)
		return
	}

	policies := make([]httpx.RoutePolicy[I, O], 0, len(r.config.policies)+len(r.config.operations))
	policies = append(policies, r.config.policies...)
	for _, operation := range r.config.operations {
		policies = append(policies, httpx.PolicyOperation[I, O](operation))
	}
	httpx.MustGroupRouteWithPolicies(group, r.method, r.path, r.handler, policies...)
}
