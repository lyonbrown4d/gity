<div lang="zh-CN">

[简体中文](components.md) · [English](../../en/architecture/components.md)

# 组件职责与依赖

## 可部署应用

| 应用 | 职责 | 主要依赖与边界 |
| --- | --- | --- |
| Web | 页面、状态、编辑器绑定与语言切换 | Leptos/WASM；调用 HTTP API；静态资源由平台提供 |
| Standalone | 组合平台、权限服务、软件包、OCI、Runner 生命周期 | 默认 SQLite、本地 Git/对象目录；Runner 使用外部容器引擎 |
| API | 身份、组织项目、协作、CI 控制 API、静态 Web、内部权限服务 | 数据库、Git 后端、对象存储；默认不监听 SSH，不运行 CI scheduler |
| Git server | Git 协作服务和 SSH 认证、权限与分支规则 | 共享数据库、Git cluster；仍运行 scheduler 和 maintenance，不是纯 SSH 转发器 |
| Git node | 保存裸仓库，执行 Git 操作、流式传输、复制和旧主写入拒绝 | 本地仓库卷、内部 RPC 凭据与 TLS；不向公网提供产品 API |
| Gateway | HTTP 分流、SSH TCP 转发、并发与连接限制 | HTTP 到 API / 软件包 Registry；SSH 到 Git server；不维护独立账户权限库 |
| Registry | 多语言包协议、Generic 包、OCI 鉴权入口和 mirror | 制品元数据库、对象存储、API Authority RPC；监督 Angos 子进程 |
| Worker | 流水线调度、产物清理和搜索维护 | 共享数据库、Git 后端、对象存储；不执行用户脚本 |
| Runner | 领取任务、取源码、执行脚本、回传日志和产物 | 服务端任务 API、项目 Runner token、容器引擎；不承担服务端调度 |
| Git hook | `receive-pack` 更新钩子适配 | 原生 Git 与服务端规则，不是常驻服务 |
| Search index | 索引维护命令 | 复用搜索和平台配置，不是必须独立部署的搜索集群 |
| Server | 通用平台命令入口 | 薄封装复用平台；新部署优先使用明确角色入口 |

“独立部署”表示进程和镜像可以分开，依赖表中的数据库、RPC 或存储条件仍须满足。Git server 与 Worker 当前均运行调度/维护；不要把 Worker 理解成唯一调度者。

## 可复用库

| 组件库 | 作用 |
| --- | --- |
| `gity-platform` | 平台 HTTP/SSH、路由适配、FluxDI 应用装配及运行时编排 |
| `gity-projects` / `gity-collaboration` / `gity-ci` | 项目与仓库访问、Issue/MR 等协作、流水线和任务应用服务及类型化仓储 |
| `gity-persistence` | 跨上下文的类型化审计快照、IID 计数和模型 JSON 边界转换 |
| `gity-identity` / `gity-authority` | 身份仓储与服务；内部 OCI/软件包权限 RPC、远程授权适配 |
| `gity-artifacts` / `gity-package-registry` | 制品规则、元数据和存储服务；各语言 HTTP 协议适配 |
| `gity-oci` / `gity-runtime` | OCI 策略与 mirror；Angos 和组合 module 生命周期 |
| `gity-git` | 本地 Git 服务和可替换后端接口；常规操作使用 git2，pack 等调用原生 Git |
| `gity-git-cluster` | 仓库放置、主副本选择、复制推进与受控提升 |
| `gity-git-node` / `gity-git-wire` | Git 节点实现；内部 RPC、客户端、流式传输与 TLS |
| `gity-gateway` | HTTP / SSH 网关 module |
| `gity-core` / `gity-entities` / `gity-ids` | 数据库连接与迁移、基础配置和存储；实体映射；统一 ID 生成 |
| `gity-search` / `gity-i18n` / `gity-observability` | 搜索、共享语言资产、日志/指标/追踪 |
| `gity-test-support` | 可回收的容器集成测试依赖 |

## 谁执行鉴权

Gateway 保留认证头与公开 Host/协议信息，转发请求。普通 API 和 Git 服务执行服务端权限与分支规则；SSH 认证在 Git server 中完成。

独立 Registry 使用 `RemotePackageAuthority` / `RemoteOciAuthorization` 向 API 内的 Authority 查询权限。Authority 不可用时请求失败，不降级为放行。上游镜像仓库凭据由 Registry 单独配置，不把用户的 Gity 令牌作为上游凭据。

Standalone 的软件包和 OCI 模块直接使用平台权限适配，不必为进程内调用绕行远程 RPC。Git node 的内部 TLS/peer 凭据约束可信组件访问，不能替代产品入口的用户权限检查。

源码依据：[Gateway HTTP](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-gateway/src/http_proxy.rs)、[Registry 组合](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-registry/src/main.rs)、[Package module](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-package-registry/src/module.rs)、[应用角色](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-platform/src/application.rs)。

</div>
