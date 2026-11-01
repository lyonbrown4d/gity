<div lang="zh-CN">

[简体中文](contributing-docs.md) · [English](../en/contributing-docs.md)

# 文档维护

当前手册统一使用 mdBook，中文正文在 `docs/zh/`，英文正文在 `docs/en/`，共用根目录 `book.toml` 和 `docs/SUMMARY.md`。图表直接使用章节中的 Mermaid 代码块，通过固定版本 mdbook-mermaid 渲染。本地资源在 `docs/shared/vendor/`，不依赖 CDN 或 Node 构建。

## 构建和预览

从仓库根目录执行：

```sh
cargo install mdbook --version 0.5.4 --locked
cargo install mdbook-mermaid --version 0.17.1 --locked
cargo xtask docs
cargo xtask docs --serve
```

输出为 `target/book/`。预览监听 `http://127.0.0.1:3000`，中文入口 `/zh/index.html`，英文入口 `/en/index.html`。也可以直接运行 `mdbook build` 或 `mdbook serve --hostname 127.0.0.1`。

CI 构建文档并保存 HTML 产物；这不代表已部署公共站点。

## 同步规则

- 新增或修改章节时同步中英文正文、目录和图表。两种语言使用相同相对路径和章节结构。
- 代码、配置键、命令和示例字面值保持语义一致；说明文字与图表标签分别使用对应语言。
- 部署和构建目录的 Markdown 只保留语言入口，不维护重复正文。
- 书内使用相对链接；源码链接指向 `main` 中实际存在的文件。调整标题后核对跨章节锚点。
- 当前手册只描述当前实现及操作。组件职责和启动顺序须对照入口、module 和 Compose 配置，不用产品目标替代已有机制。
- 验证状态单独记录已检查和未覆盖范围。文档构建不会执行部署、数据库更新、流水线或压力测试。

配置与目录语法参见 [mdBook 官方配置](https://rust-lang.github.io/mdBook/format/configuration/general.html)和[目录说明](https://rust-lang.github.io/mdBook/format/summary.html)。

资源来源、许可证与 SHA-256 在 [manifest](../shared/vendor/manifest.toml) 中记录；`cargo xtask docs` 在构建前校验。

</div>
