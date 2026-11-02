# Gity 工程约定

## Rust 主线与工作分支

- `main` 为唯一主维护分支，服务端、Runner 和 Web 全部以 Rust 版本为准。
- Rust 整体迁移与本机验收已完成；后续按用户授权直接在 `main` 迭代，完成验证后提交，不能把未完成的变更当作可发布版本。
- Go 实现、Go 模块与 Go 构建发布配置已删除，不得重新引入；Web 前端已切换回 React 生态，历史版本可从 Git 提交记录追溯。
- 用现有业务流程、权限规则和 API 使用场景核对迁移完整性；数据结构以最新 Rust 版本为准，不能以返回空数据或无操作成功替代功能。
- 允许多个 agent 并行落地，但必须明确互不冲突的文件归属，由主 agent 集成并复核结果。

## 目录与依赖方向

- `apps/`：可打包、部署的应用，例如 server、runner 和 Web。
- `crates/`：共享领域、应用服务和基础设施库，不混入应用可执行入口。
- `xtask/`：使用 Rust 实现的仓库构建、验证和资源检查工具。
- `crates/gity-migrations/`：使用 SeaORM 官方 `sea-orm-migration` 管理版本化 Rust 迁移，支持 SQLite、PostgreSQL 和 MySQL；结构优先使用 SeaQuery 与 `DeriveIden`，不得另建迁移执行器或运行时 SQL 解析器。
- 迁移版本必须固定结构快照，不从可变的当前 Entity 自动推导历史版本；已发布的 Rust 迁移保持不变，后续变更新增版本。迁移由 standalone 启动入口或分布式部署的单一迁移任务调用，不能让多个写入组件并发迁移。
- 数据结构以最新 Rust 版本为唯一基准，不接管旧 Go 数据库，不为历史 schema 保留兼容分支。使用最新初始 schema，并继续通过版本化迁移维护后续 Rust 版本。
- 应用依赖共享库；共享库不能反向依赖应用。
- 不把 HTTP、数据库实体和前端组件混成同一个层次；公共请求响应类型不得包含服务端秘密。

## DDD、实体与组件部署

- 按身份权限、组织项目、Git仓库、协作、CI、制品等限界上下文组织领域与应用服务；领域层不依赖HTTP、SeaORM或RPC。
- HTTP/平台适配层通过 FluxDI 注入的应用服务处理业务，不直接操作 Entity/ActiveModel 或执行 SQL；应用服务接收类型化命令，仓储负责类型化查询、事务和原子条件更新。
- `gity-core` 的 `test-fixtures` 功能仅供测试依赖显式启用；生产依赖不得启用，也不得调用其中的通用 SQL/JSON 夹具。提交前同时检查默认工作区构建与测试构建，避免测试功能掩盖生产依赖错误。
- 仓储适配优先使用SeaORM Entity、ActiveModel、类型化列和事务；禁止新建表名/字段名加JSON的通用CRUD或在业务层新增手写SQL。迁移DDL及必要的数据库专属操作须明确理由。
- 组件共享业务实现，通过进程内或RPC适配器切换部署；standalone默认包含全部组件，另提供Git协作等有限官方镜像变体。
- Git基础分布式与Raft高可用模式分别验证；不能将异步副本、进程数或Git mirror当作已实现无损自动故障切换。
- 制品包含多语言包、Generic包和Angos OCI；OCI mirror为受控拉取缓存，上游凭据与本地身份分开。

## 分布式标识

- 数据库实体主键固定采用应用生成的雪花 ID，禁止自增、数据库 sequence 或数据库默认 ID。所有 SeaORM 主键声明 `auto_increment = false`，新增实体通过统一生成器分配 ID。
- 算法策略在代码中固定，不提供切换到自增的配置；分布式部署的各写入进程必须配置唯一的 `database.node_id`，standalone 的组件共享同一生成器。
- 非实体操作、请求和事件标识使用 UUID v7。密码、访问令牌和其他秘密使用密码学随机数，不使用可预测标识替代。
- HTTP/JSON 对外将 i64 主键序列化为字符串，避免浏览器整数精度丢失。结构性键（迁移版本、翻译键、枚举代码）按自身语义建模。

## Rust 技术栈

- 当前只交付 Web，不实现桌面端及桌面本地仓库管理。
- 后端采用 Rust、Axum、Tokio、SeaORM。
- Web 页面、状态和业务交互采用 React 19、TypeScript 与 Vite；后端保持 Rust。
- Rust 工具链版本必须满足 FluxDI 要求，固定在 `rust-toolchain.toml`。
- 依赖注入统一使用 `fluxdi`：<https://github.com/lyonbrown4d/fluxdi>。
- 程序组合入口通过 FluxDI 注册和解析依赖，不能只添加依赖却绕过容器装配。
- 各可部署组件导出 FluxDI module；standalone 通过导入这些 module 组合全功能应用，并使用 FluxDI 生命周期管理启动、健康检查和关闭。组件不通过依赖其他 app 复用实现。
- FluxDI 出现可复现的能力缺口时，先核对文档与上游 issue；用户已授权使用 gh 向上游提交问题，并附最小复现。不得仅因尚未熟悉 API 自建第二套 DI 容器。
- 命令行统一使用 Clap 的派生宏和类型化参数。
- 优先采用成熟第三方库；优先使用 Serde、Clap、FluxDI 等库提供的派生宏或声明式宏减少样板代码。
- 宏必须保持可读的依赖关系和错误提示，不为节省几行代码自建复杂宏系统。
- 服务端使用成熟配置库，支持配置文件、环境变量和明确默认值，并在启动时验证。
- 各可部署应用统一配置优先级：代码默认值 < 配置文件 < 环境变量 < 显式 CLI 参数；配置项不得仅支持 CLI。对错误配置给出不含秘密值的启动错误。
- 日志使用 Tracing；提供请求上下文、结构化日志、Prometheus 指标和可选 OTLP 追踪导出。
- 不记录密码、原始令牌、私钥、完整认证头或 CI 秘密变量。
- Swagger UI 与 OpenAPI 文档端点仅用于显式启用 `api-docs` 的开发构建；release 二进制、镜像和 Web 静态产物不得提供或包含 Swagger UI 资源、`/docs`、`/openapi.json` 等文档端点。保留成熟宏用于开发及离线契约生成，不能因此移除 Git LFS 等真实业务协议路由。
- 接口文档须覆盖实际路由、请求响应、认证与错误结构，随实现同步验证，不能只提供空壳页面。
- Utoipa 优先使用 `ToSchema`、`OpenApi` 派生宏、`#[utoipa::path]` 和 `utoipa-axum::routes!`；优先关联路由注册与文档收集，避免手写重复接口清单。
- 集成环境优先使用 Docker Compose，覆盖数据库、对象存储及可观测性依赖；提供健康检查、持久化卷、环境变量示例和可复现的启动与验证说明。
- 分别提供 standalone 与分布式组件模式的 Docker Compose 部署文件；当前都以单机 Docker Compose 为目标。默认 standalone 包含全部业务组件，各角色同时提供 Debian 与 Alpine 基础镜像变体。
- 镜像黑盒分别验收两种运行架构，覆盖真实 SSH Git、包协议、OCI/mirror、鉴权隔离及故障行为；不能以单体通过推断分布式通过，也不能以 Debian 通过推断 Alpine 通过。
- S3 集成环境使用 SeaweedFS；应用通过标准 S3 API 访问，验证鉴权、对象读写、Range 和分段上传。
- i18n 语言资产必须在编译期嵌入后端 Rust 二进制和前端构建产物；运行时不得依赖外部语言文件、语言目录挂载或 CDN 下载。
- 前后端复用 `crates/gity-i18n` 的多语言资产、翻译键与参数定义。前端支持语言切换及偏好保存，服务端按请求选择语言；禁止通过全局可变 locale 处理并发请求。验证语言包键和占位符一致性。

## 多协议 API

- REST、GraphQL 和 MCP 复用相同应用用例、授权和审计，适配层不直接操作 ORM 或创建第三套业务流程。
- GraphQL 使用 async-graphql，MCP 使用官方 Rust SDK rmcp；优先复用现有宏产出的类型契约并验证操作覆盖，不依赖运行时 Swagger 文档端点。
- MCP 使用 Streamable HTTP，按请求验证凭据、scope、Origin 和资源权限；不得将会话标识当作授权，不得缓存已过期的管理权限。
- 管理员由配置引导的一次性事务初始化，普通注册永远不能自动取得管理员权限；配置变更和重启不覆盖已初始化账号的密码和权限。

## 前端工具链与第三方编辑器

- Web 使用 React 19、TypeScript、Vite 构建，包管理使用 pnpm 并以锁文件固定版本；不引入 Trunk 或 Leptos。
- 前端产物由 Vite 构建到 `dist/web`，再由 `rust-embed` 编译进后端二进制；release 构建顺序必须是先构建前端、再编译 Rust。
- pnpm 使用全局 store，不在仓库内创建 `.pnpm-store`；store 与安装方式固定在 `web/pnpm-workspace.yaml`。
- 可重建的生成物不提交仓库，包括 `web/openapi.json` 与 `src/lib/generated/`；接口类型与 React Query 客户端通过 `pnpm gen:api`（Orval）从后端 OpenAPI 生成。
- 前端依赖更新到最新版本；破坏性升级由前端适配，不使用长期滞后的锁版本。
- `src/components/ui/` 是 shadcn 生成产物，不得手改；定制通过 `src/components/` 下的组合组件实现，升级 shadcn 时直接覆盖 `ui/`。
- 样式优先使用 Tailwind CSS；自定义样式使用 SCSS（`src/index.scss`），构建链为 sass + PostCSS。
- 编辑器使用 Monaco 与 Milkdown（npm 依赖），不再维护手工 vendor 的第三方资产；不依赖 CDN。
- 动效使用 motion，并尊重 `prefers-reduced-motion`；动效不得改变原有布局与视觉规范。
- 前端优先复用成熟第三方组件与浏览器生态，避免自行重写已有的通用能力。
- 按组件组织页面；表单、列表、分页、弹窗、状态反馈和编辑器应通过明确的组件接口复用。
- 业务权限、数据请求和展示组件职责分开，不为不同页面复制同一组通用交互代码。
- UI 迁移必须完整还原原有样式、布局、排版及交互状态，不能以通用管理界面替代原产品。
- 以迁移前版本的实际页面和源码为视觉基准，核对导航、间距、字体、颜色、表格、弹窗、响应式和主题。
- 复用第三方组件时必须适配原有视觉规范，不能因组件库默认主题改变现有产品外观。
- 代表性页面完成浏览器视觉对照后才能认定 UI 变更完成；功能通过不能替代视觉验收。

## Git 与 CI

- 常规仓库操作优先使用 `git2/libgit2`；仅在明确有收益时引入其他 Git 库。
- 服务端允许调用原生 Git 处理 upload-pack、receive-pack、维护和完整性检查。
- Git SSH 使用成熟 Rust 第三方库独立监听，监听地址、端口、主机密钥及资源限制可配置；必须用真实 Git/SSH 客户端验证 clone、push、pull，以及密钥隔离与分支保护。
- 子进程直接传递参数，不能拼接用户输入为 shell 命令。
- Git HTTP、API 写入和 MR 合并均须执行服务端权限与分支保护规则。
- 保留流式传输、取消、资源限制和进程回收，不能把大仓库完整缓存在内存中。
- CI 配置采用 YAML，默认文件为 `.gity-ci.yml`。
- 不迁移或新建 Plano 等自定义 DSL；YAML 必须有类型、校验及明确的错误提示。
- Runner 与服务端调度职责分开，服务端不直接执行用户流水线脚本。

## 文件规模与测试

- Rust 使用 2 空格缩进，由根目录 `rustfmt.toml` 固定 `tab_spaces = 2` 和 `hard_tabs = false`；提交前使用 Cargo fmt 统一格式，并与 `.editorconfig` 保持一致。

- 每个手写实现文件最多 300 行，每个测试文件也最多 300 行。
- 实现与测试必须分开存放；实现文件不能包含内联测试函数或测试模块体。
- 可通过 `#[cfg(test)] mod tests;` 引用独立测试文件，或使用 crate 的 `tests/` 目录。
- 达到限制时按职责拆分，不能删除测试、压缩多条语句或用超长行规避限制。
- Cargo 锁文件、生成产物和保留原貌的第三方分发资产不手工拆分为源码模块。
- 验证应覆盖真实行为、失败路径、权限隔离、并发和数据一致性。
- 容器集成依赖优先用 Rust Testcontainers 管理独立生命周期和清理；纯逻辑单元测试不强制启动 Docker。容器测试不得复用或清理用户保留的检查环境数据。
- 报告必须区分源码检查、已编译、已运行测试和未验证平台，不能把构建成功等同于功能验收。
- Go 遗留删除前核对迁移功能；删除后重新运行 Rust、Web 和发布相关验证。

## 项目文档

- 项目手册统一使用 mdBook，中英文正文分别在 `docs/zh/`、`docs/en/`，由根目录 `book.toml` 与 `docs/SUMMARY.md` 管理。
- 部署和构建目录的 Markdown 仅保留指向双语正文的入口，不维护重复内容。
- 当前手册只描述最新实现和操作，不编入旧实现对照、迁移故事或历史规划；章节和图表须保持中英对应。
- 组件职责、依赖和启动行为变更时同步架构章节；区分当前实现、历史计划及实际验证范围。
- 使用 `cargo xtask docs` 构建、`cargo xtask docs --serve` 预览；保持纯 Rust 文档工具链。
- 架构和数据流图优先使用章节内 Mermaid，由固定版本 `mdbook-mermaid` 渲染；使用本地资源并记录来源、许可证与校验值。
