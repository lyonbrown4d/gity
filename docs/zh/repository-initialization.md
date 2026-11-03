# 仓库初始化选项

创建项目时可以选择 License、`.gitignore`，并勾选“添加 README 初始化仓库”。这些选项独立生效；选中的文件会在指定默认分支上组成一次初始提交。全部不选时创建空仓库，不生成提交。

README 使用项目名称和描述生成，按普通文字转义 Markdown 特殊字符。License 写入根目录的 `LICENSE`；`.gitignore` 提供来自 [github/gitignore](https://github.com/github/gitignore) 的 13 个候选：Rust、Node.js、Python、Go、Java、C++、Visual Studio/.NET、Dart、Swift、Kotlin、Ruby、Unity、Android。模板采用固定提交，保留 CC0-1.0 授权文本，并在 `provenance.json` 中记录来源与 SHA-256。

## 许可证模板

| 模板 ID | 许可证 |
| --- | --- |
| `mit` | MIT |
| `apache-2.0` | Apache License 2.0 |
| `bsd-2-clause` | BSD 2-Clause |
| `bsd-3-clause` | BSD 3-Clause |
| `mpl-2.0` | Mozilla Public License 2.0 |
| `gpl-3.0-only` | GNU GPL v3.0 only |
| `agpl-3.0-only` | GNU AGPL v3.0 only |
| `unlicense` | Unlicense |

MIT 和 BSD 模板填入当前 UTC 年份与版权所有者，默认使用所属组织的名称，可在表单中修改。其他许可证保留官方文本及附录示例。历史 ID `gpl-3.0` 仍按 GPL-3.0-only 处理。

文本来自固定版本的 [SPDX License List Data v3.27.0](https://github.com/spdx/license-list-data/tree/v3.27.0/text)，通过 `include_str!` 编译到应用中，创建项目不需要联网。原始文件、来源 URL 和 SHA-256 位于 `crates/gity-projects/templates/` 与其中的 `provenance.json`。

## API

`GET /api/v1/repository-templates/licenses` 返回可选模板的 `id`、`name` 和 `spdx_id`。`GET /api/v1/repository-templates/gitignores` 返回 `.gitignore` 模板的 `id` 与 `name`。Web 下拉菜单使用这两份列表，分别处理加载、失败和重试，仍可选择不添加模板。

`POST /api/v1/projects` 在原有创建参数上支持：

```json
{
  "organization_id": "123456789012345678",
  "name": "Example",
  "key": "example",
  "default_branch": "main",
  "initialize_with_readme": true,
  "license_template": "mit",
  "license_copyright_holder": "Example Contributors",
  "gitignore_template": "rust"
}
```

`initialize_with_readme` 默认 false；模板字段省略、空值或 `none` 表示不添加。`license_copyright_holder` 只作用于 MIT/BSD。未知模板和错误参数在创建项目之前返回 400；创建仍要求组织 Developer 及以上权限。初始化 Git 存储失败时，服务回滚项目记录并返回 503。

本地与 Git 节点 RPC 使用同一初始化实现，一次写入文件树并创建根提交，不改写已有仓库。节点在完成提交前保留初始化状态；恢复时核对分支和初始内容摘要，不覆盖不同的已有提交。初始化不触发 CI 流水线，搜索投影通过既有事件和周期核对更新。
