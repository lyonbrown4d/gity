<div lang="en">

[简体中文](../../zh/architecture/components.md) · [English](components.md)

# Component responsibilities and dependencies

## Deployable applications

| Application | Responsibility | Dependencies and boundaries |
| --- | --- | --- |
| Web | Pages, state, editor bindings, and language selection | Leptos/WASM; calls HTTP APIs; the platform serves its static assets |
| Standalone | Composes platform, authority, packages, OCI, and Runner lifecycle | Supplied defaults use SQLite, local Git and object directories; Runner requires a container engine |
| API | Identity, organizations/projects, collaboration, CI control APIs, Web, internal authority | Database, Git backend, object storage; no SSH listener or CI scheduler by default |
| Git server | Git collaboration, SSH authentication, permissions, branch rules | Shared database and Git cluster; also runs scheduler and maintenance |
| Git node | Bare repository storage, Git operations, streaming, replication, rejection of fenced writes | Repository volume, internal RPC credentials and TLS; no public product API |
| Gateway | HTTP routing, SSH TCP forwarding, connection and concurrency limits | Forwards HTTP to API/package Registry and SSH to Git server; no independent identity database |
| Registry | Language package protocols, Generic packages, OCI authorization and mirror | Artifact metadata database, object storage, API Authority RPC; supervises Angos |
| Worker | Pipeline scheduling, artifact cleanup, search maintenance | Shared database, Git backend and object storage; does not execute user scripts |
| Runner | Claims jobs, fetches sources, executes scripts, uploads logs and artifacts | Task API, project Runner token and container engine; does not schedule server jobs |
| Git hook | Native `receive-pack` update-hook adapter | Native Git and server rules; not a persistent service |
| Search index | Index maintenance command | Reuses search and platform configuration; not a separate search cluster |
| Server | General platform command entry point | Thin wrapper around platform code; deployments normally select a specific application role |

Independent deployment means separate processes and images, with the dependencies above still required. Git server and Worker both run scheduling and maintenance today; Worker is not the sole scheduler.

## Reusable libraries

| Libraries | Responsibility |
| --- | --- |
| `gity-platform` | Platform HTTP/SSH, route adapters, shared API operation catalog, FluxDI composition and runtime orchestration |
| `gity-api-contracts`, `gity-api-macros` | Request context, operation/error contracts, macro extraction of shared application entry points |
| `gity-graphql`, `gity-mcp` | async-graphql and rmcp adapters sharing use cases, authorization, and audit |
| `gity-projects`, `gity-collaboration`, `gity-ci` | Project/repository access, Issue/MR collaboration, and pipeline/job application services with typed repositories |
| `gity-persistence` | Typed cross-context audit snapshots, IID counters, and model-to-JSON boundary conversion |
| `gity-identity`, `gity-authority` | Identity repository/services; internal OCI/package authorization RPC and remote adapters |
| `gity-artifacts`, `gity-package-registry` | Artifact rules, metadata/storage services, language HTTP protocol adapters |
| `gity-oci`, `gity-runtime` | OCI policy and mirror; Angos and composition module lifecycle |
| `gity-git` | Local Git services and replaceable backend; git2 for normal operations, native Git for pack operations |
| `gity-git-cluster` | Placement, primary/replica selection, replication, controlled promotion |
| `gity-git-node`, `gity-git-wire` | Storage node; internal RPC, clients, streaming and TLS |
| `gity-gateway` | HTTP/SSH gateway module |
| `gity-core`, `gity-entities`, `gity-ids` | Database connection and migrations, configuration and storage; entity mappings; shared ID generation |
| `gity-search`, `gity-i18n`, `gity-observability` | Search, shared language assets, logging/metrics/tracing |
| `gity-test-support` | Disposable integration container support |

At API startup, `PlatformModule` registers the operation service, GraphQL module, and MCP module in the same FluxDI container and merges their routes into the platform listener. Identity also supplies persisted OAuth services; see [Administration and MCP authorization](../reference/administration.md) for consent and bootstrap settings.

## Authorization ownership

Gateway preserves credentials and public Host/scheme information while forwarding requests. Product services enforce API permissions and branch rules. Git server performs SSH authentication.

An independent Registry uses `RemotePackageAuthority` and `RemoteOciAuthorization` to query Authority in API. An unavailable Authority causes failure, not authorization bypass. Upstream mirror credentials are configured separately from the caller's Gity token.

Standalone package and OCI modules use local platform authorization adapters. Git node TLS and peer credentials restrict trusted component access; they do not replace product-level user authorization before internal calls.

Sources: [Gateway HTTP forwarding](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-gateway/src/http_proxy.rs), [Registry composition](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-registry/src/main.rs), [Package module](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-package-registry/src/module.rs), [application roles](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-platform/src/application.rs).

</div>
