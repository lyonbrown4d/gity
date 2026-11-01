<div lang="en">

[简体中文](../zh/index.md) · [English](index.md)

# Gity technical manual

Gity uses Rust for Git hosting, project collaboration, CI, packages, and OCI registries. Web uses Leptos and WebAssembly. The current product delivers a Web application.

- Understand the system: [architecture](architecture/overview.md), [components and dependencies](architecture/components.md), [data flows](architecture/data-flows.md).
- Understand startup: [Standalone](architecture/standalone.md), [distributed components](architecture/distributed.md).
- Deploy: [Compose operations](deployment/compose.md), [image variants](deployment/images.md), [deployment boundaries](production-deployment.md).
- Develop and use: [CI YAML](ci-yaml.md), [Runner](runner-container-runtime.md), [Git SSH](git-ssh.md), [localization](i18n.md).
- Check scope: [product scope](roadmap.md), [verification status](verification.md), [release checks](release-checklist.md).

This manual describes the current `main` implementation. Compose targets one Docker host. Git uses asynchronous replicas and controlled promotion; consult verification status for actual image acceptance coverage. See [documentation maintenance](contributing-docs.md) for tooling and bilingual updates.

</div>
