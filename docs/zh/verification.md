<div lang="zh-CN">

[简体中文](verification.md) · [English](../en/verification.md)

# 当前验证状态

本页分别记录源码检查、编译、运行测试和镜像验收；构建通过不等同于完整功能验收。

## 源码与编译检查

- 默认工作区 `cargo check --workspace --locked` 通过，生产依赖不启用 `gity-core/test-fixtures`。
- Native 工作区 `cargo clippy --workspace --all-targets --locked -- -D warnings` 通过。
- Web 的 `wasm32-unknown-unknown` 目标检查通过。
- 格式、单文件 300 行、实现与测试分离检查通过；源码约束禁止平台业务适配层使用通用 Db CRUD、原始 SQL 或直接操作 Entity/ActiveModel。
- mdBook 构建、双语章节配对、本地链接与锚点、Mermaid 本地资产校验通过。

固定工具链下检查通过。Cargo 仍提示第三方 `proc-macro-error2 v2.0.1` 存在未来 Rust 兼容性风险。

## 已运行测试

工作区测试 **259 项通过、8 项默认忽略**。覆盖类型化仓储、项目隔离、审计与业务写入共同回滚、秘密字段排除、并发编号分配、流水线去重、Runner 租约、LFS 并发上传，以及真实 Git HTTP/SSH 的权限和分支保护行为。

独立 PostgreSQL 18 与 MySQL 8.4 Testcontainers 分别执行平台和服务端回归集合，**各 71 项通过**。测试通过 `GITY_TEST_DATABASE_URL` 切换可配置的数据库夹具；纯逻辑测试和自建 SQLite 的传输测试仍使用各自资源，不表示每一项都切换了数据库后端。所有容器和测试库均为临时创建，不使用保留的检查环境数据。

运行双数据库集合：

```sh
cargo test -p gity-test-support --test database_regression --locked -- --ignored --test-threads=1
```

默认忽略的两项数据库回归入口已单独运行。其余六项专项测试未在本次业务持久化检查中重新执行；它们包括实体与迁移容器契约、OCI 上游解析及容器 Runner 检查。默认忽略不代表失败或通过。

## 未验证范围

本次未重建 Debian/Alpine 应用镜像，也未完成 standalone/distributed × Debian/Alpine 四组完整镜像黑盒。数据库容器回归不能替代镜像、OCI/mirror 或分布式故障验收。操作入口见[镜像黑盒说明](deployment/images.md#镜像黑盒验收)。

未测量稳态 CPU、峰值内存或吞吐量；未验证 arm64、多主机网络及生产备份恢复。

</div>
