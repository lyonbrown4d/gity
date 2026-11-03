# 全局代码与议题搜索

登录用户通过 `POST /api/v1/search` 搜索当前有权读取的项目。代码与议题分组返回，支持按组织、项目和资源类型缩小范围。搜索匹配大小写不敏感的 Unicode 子串；代码索引包括默认分支的文件路径和 UTF-8 文本，议题索引包括标题和描述。二进制文件、大于 `search.max_file_size_bytes` 的文件不进入代码索引。

## 权限与结果

每次请求先从数据库读取当前账号状态、项目和组织成员关系。公开、内部项目遵循现有项目读取规则，私有项目要求当前成员权限。保密议题只对当前项目 Reporter 及以上、作者、指派人和有效管理员可见；作者和指派人仍须能够读取项目。项目访问令牌不能继承创建者对保密议题的访问权，也不能调用用户全局搜索。

引擎查询在项目、已发布索引版本和保密议题范围内执行。服务随后回查议题或固定提交下的 Git 文件，校验内容版本，并在返回前再次检查身份和权限。成员撤销、令牌撤销、议题变为保密及指派变更不等待索引更新才生效。旧待办、动态与订阅列表同样重新检查当前权限。

结果没有未授权资源的总数。`snippet` 是纯文本，客户端不得作为可信 HTML 渲染。代码结果携带项目、文件路径、提交 SHA 和行号；每个文件返回一个命中。议题结果携带项目内 IID。

## 请求与分页

```json
{
  "query": "登录失败",
  "types": ["code", "issue"],
  "project_ids": ["123456789012345678"],
  "limit": 20
}
```

Web 快捷搜索每组显示前 8 条结果，有更多命中时会提示细化查询；API 支持下述游标分页。

省略 `project_ids` 时搜索全部可读项目；`organization_id` 可进一步限定组织。`limit` 默认 20，范围 1–50，分别作用于每组。查询去除首尾空白后必须有内容，UTF-8 长度最多 512 字节。

响应中的 `groups` 包含 `kind` 与 `items`。继续分页时，将 `next_cursor` 作为 `cursor`，其余请求字段保持一致。游标最长有效 120 秒，绑定用户、查询和权限范围；范围变化、游标失效或请求字段变化返回 409，客户端应重新查询。权限撤销后不能用旧游标取得旧结果。令牌失效返回 401。

游标快照缓存在当前 API 进程，最多 128 份、每用户 8 份，并受 64 MiB 估算内存上限约束；压力下可提前失效。多个 API 副本需要会话黏性路由来继续同一游标，否则按 409 重新查询。

单次候选快照每组最多 1000 条；客户端应提示 `partial: true`，并引导用户缩小范围。引擎扫描达到预算或项目尚未发布索引时也会标记部分结果。`indexing_eventually_consistent: true` 表示新增内容异步可见，不能据此把零命中解释为资源一定不存在。源数据已修改的旧命中会被跳过，等重建后重新可见。每次搜索最多 30 秒；并发槽满、引擎或 Git 源不可用返回 503。

## 选择引擎

单进程 standalone 默认使用内嵌 Tantivy，不需要额外服务：

```toml
[search.global]
enabled = true
backend = "tantivy"
timeout_seconds = 10
```

索引保存在 `search.index_root/global-v1`，应与该实例一起持久化。该目录通过进程锁防止同时打开。内嵌模式只支持一个应用进程持有该数据库的全局搜索投影，不支持多个独立本地目录轮流发布同一数据库。API、Git、Worker 组件入口会拒绝启用 Tantivy 的配置。

分布式组件和多个 API 副本使用同一个 OpenSearch 集群与索引：

```toml
[search.global]
backend = "opensearch"
endpoint = "https://search.example.internal:9200"
index = "gity-global-v1"
timeout_seconds = 10
```

通过 `GITY_SEARCH__GLOBAL__USERNAME`、`GITY_SEARCH__GLOBAL__PASSWORD` 注入凭据。端点不得包含 URL 用户名、密码、查询参数或片段；HTTPS 使用系统信任的证书。客户端禁止重定向。远端只保存搜索投影，不能替代业务数据库授权。远端索引包含源码和保密文本，应限制网络入口及服务账号的索引访问权限。

分布式 Compose 提供内部 OpenSearch 服务，固定镜像为 `public.ecr.aws/opensearchproject/opensearch:3.9.0`，数据保存在 `opensearch-data`，9200 端口不发布到宿主机。设置 `OPENSEARCH_INITIAL_ADMIN_PASSWORD` 后，API、Git 和 Worker 共用该服务。示例显式设置 `allow_insecure_http=true`，通过内部 HTTP 与 Basic Auth 连接；跨主机部署请改为受信任的 HTTPS 端点与独立服务账号。

索引使用固定的 Gity 映射版本；不兼容的现有索引会拒绝查询和写入。请配置新的 `index` 名称，让后台重新生成投影，不要复用其他系统的索引。

配置优先级与应用一致：默认值、配置文件、环境变量、显式 CLI 参数。`enabled=false` 关闭全局搜索，接口返回 409，后台不执行全局重建。

## 索引更新与恢复

议题创建、修改与待重建任务在同一数据库事务中提交。Git 写入事件标记项目待重建；后台还会在启动时和 `search.refresh_interval_seconds` 周期内核对全部项目，修复遗漏事件。代码读取通过现有 Git 适配器，既支持本地仓库，也支持 Git 节点 RPC。

任务按项目合并并递增代数，以 300 秒租约领取。后台为每次领取创建独立命名空间，完整写入成功后，再通过数据库条件更新发布。旧 Worker 即使晚完成也不能替换新发布。失败保留上次成功版本，延迟重试；不存在空操作成功。更换索引卷或远端配置后，新实例不会接受其他索引实例的发布标识，启动核对后重建。

项目重建上限为 100000 个议题、20000 个 Git 树条目，以及源码与议题正文合计 256 MiB；代码文件数还受 `search.max_files_per_project` 限制。超限或单次重建超过 120 秒会保留旧版本并重试；此时需调整项目内容或配置预算。单个大文件和非文本文件按上述规则跳过。OpenSearch 的每条编码文档还受 5 MiB 写入上限约束，包含正文和检索字段；超限时保留旧版本并记录失败。`gity_global_search_jobs_total{result="success|failure"}` 与结构化日志用于定位重建失败；日志不记录查询正文或凭据。

实现位置：`crates/gity-search` 管理两个引擎，`crates/gity-platform/src/global_search` 编排权限、投影、查询和分页，`gity-persistence::search_jobs` 管理任务。验证命令为 `cargo test -p gity-search`、`cargo test -p gity-persistence --test search_jobs` 及 `cargo test -p gity-platform global_search`；远端实测要求独立 OpenSearch 测试实例，不能用编译成功代替远端验收。
