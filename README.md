# Gity

Gity 是使用 Rust 构建的 Git 托管与协作平台，提供 Web、仓库、Issue/MR、CI、多语言包仓库与 OCI/mirror。

Gity is a Rust Git hosting and collaboration platform with Web, repositories, issues/MRs, CI, language packages, and OCI/mirror.

- [简体中文手册](docs/zh/index.md) · [English manual](docs/en/index.md)
- [架构与组件](docs/zh/architecture/overview.md) · [Architecture and components](docs/en/architecture/overview.md)
- [Compose 部署](docs/zh/deployment/compose.md) · [Compose deployment](docs/en/deployment/compose.md)

```sh
cargo install mdbook --version 0.5.4 --locked
cargo install mdbook-mermaid --version 0.17.1 --locked
cargo xtask docs
cargo xtask docs --serve
```

预览 / Preview: `http://127.0.0.1:3000`。输出 / Output: `target/book/`。
