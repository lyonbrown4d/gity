<div lang="en">

[简体中文](../zh/contributing-docs.md) · [English](contributing-docs.md)

# Documentation maintenance

The current manual uses mdBook. Chinese chapters live in `docs/zh/`, English chapters in `docs/en/`, with a shared root `book.toml` and `docs/SUMMARY.md`. Diagrams are Mermaid blocks in the chapters, rendered by a pinned mdbook-mermaid preprocessor. Assets in `docs/shared/vendor/` require no CDN or Node build.

## Build and preview

Run from the repository root:

```sh
cargo install mdbook --version 0.5.4 --locked
cargo install mdbook-mermaid --version 0.17.1 --locked
cargo xtask docs
cargo xtask docs --serve
```

Output is `target/book/`. Preview listens on `http://127.0.0.1:3000`, with Chinese at `/zh/index.html` and English at `/en/index.html`. Direct `mdbook build` and `mdbook serve --hostname 127.0.0.1` also work.

CI builds the manual and retains HTML artifacts; this does not imply public hosting.

## Synchronization rules

- Update both languages, navigation, and diagrams together. Use matching relative paths and chapter structure.
- Preserve the meaning of code, configuration keys, commands, and literal examples. Translate explanatory prose and diagram labels.
- Markdown in deployment/build directories is an entry point, not a duplicate authoritative body.
- Use relative links within the book and existing files on `main` for source references. Check cross-chapter anchors after changing headings.
- Describe the current implementation and operations. Verify responsibilities and startup against application entry points, modules, and Compose configuration.
- Record checked and uncovered behavior separately. Building documentation does not execute deployment, schema updates, pipelines, or load tests.

See the official [mdBook configuration](https://rust-lang.github.io/mdBook/format/configuration/general.html) and [summary syntax](https://rust-lang.github.io/mdBook/format/summary.html).

Asset sources, licenses, and SHA-256 values are recorded in the [manifest](../shared/vendor/manifest.toml); `cargo xtask docs` verifies them before building.

</div>
