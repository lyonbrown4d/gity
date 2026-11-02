<div lang="en">

[简体中文](../zh/roadmap.md) · [English](roadmap.md)

# Product scope

Gity provides repositories, Issues, MRs, CI, packages, and container registries for team collaboration and self-hosted deployments through the Web and standard Git clients. The server uses Rust, Axum, Tokio, and SeaORM; the Web uses Leptos/WASM; FluxDI assembles the components.

## Current boundaries

- Project memberships, organization memberships, and token scopes jointly determine effective permissions, enforced by the server.
- Bare Git repositories are located by immutable project IDs; public paths can differ from physical storage names.
- SQLite, PostgreSQL, and MySQL use versioned Rust migrations for initialization and subsequent schema updates.
- Runners are separate from server scheduling and execute project scripts on trusted Runner hosts.
- Package protocols include Generic, Cargo, npm, PyPI, Maven, NuGet, and Go. OCI uses Angos, with configuration-controlled mirror upstreams.
- Standalone combines all components. The distributed example separates processes and shares PostgreSQL and SeaweedFS, currently targeting single-host Compose.
- Git replication is asynchronous with controlled promotion; lossless automatic Raft failover is not provided.
- Offline backup tooling captures an entire stopped Compose project and restores it to a separate project using locally available original image IDs; see [production deployment](production-deployment.md).
- The current delivery targets the Web; desktop local repository management is out of scope.

See [overall architecture](architecture/overview.md) for component relationships and [verification status](verification.md) for the actual checked scope.

</div>
