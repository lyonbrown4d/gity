<div lang="zh-CN">

[简体中文](web-workspace.md) · [English](../en/web-workspace.md)

# Web 工作台与主题

Web 使用 React、React Router、Refine、shadcn 与 Tailwind CSS。工作台呈现真实待办、最近项目和动态；仓库代码页提供分支搜索、文件树与 Monaco；合并请求页面提供差异、行内讨论、审批和合并条件。

## 页面与接口

项目页面以 `/app/projects/{organizationId}/{projectId}` 为根路径。代码、议题、评审和 CI 使用各自路由，筛选条件保留在查询参数中。流水线编辑位于 `pipelines/editor`，运行详情位于 `pipelines/{pipelineId}`，任务日志位于 `pipelines/{pipelineId}/jobs/{jobId}`。独立的 `jobs/{jobId}` 也可直接访问。

页面通过 Refine 查询和修改数据。provider 匹配编译期生成的操作注册表，再调用 Orval 生成函数；唯一 Axios 传输层负责令牌、单次并发刷新、取消、语言头和 HTTP 错误。未登记操作和未知参数明确报错，不回退到另一套请求实现。

从仓库根目录运行：

```sh
cargo run -p gity-server -- openapi --output web/openapi.json
pnpm --dir web gen:api
pnpm --dir web lint
pnpm --dir web test
pnpm --dir web build
```

生成的契约、客户端和操作注册表不提交 Git。release 仍不提供 Swagger 文档端点。手写函数使用箭头函数，由 ESLint 强制；生成代码不手改。

## 用户主题

顶栏的配色设置提供浅色、深色、跟随系统，以及 Mineral、Ocean、Iris、Rose 预设。用户可修改主色、页面背景、侧栏背景和圆角；偏好即时保存到当前浏览器，可恢复默认，不代表跨设备账号同步。

shadcn、代码与差异编辑器、日志和 React Flow 使用统一语义变量。自定义主色会调整到可读的色阶，按钮前景色自动匹配；状态同时使用文字和图标。主题更新会重设已挂载的 Monaco 配色。

## CI 编辑与运行

React Flow 用任务节点和依赖连线编辑 `.gity-ci.yml`；Monaco 提供 YAML 视图。任务属性与 Rust 配置契约一致，包含阶段、脚本、镜像、依赖、变量、Runner 标签、超时和产物路径。未指定 `needs` 表示按前置阶段等待，`needs: []` 表示立即就绪；两者不会在切换视图时混淆。

校验会检查未知字段、无效依赖、循环和阶段顺序。提交配置需要分支权限，并携带读取时的提交 SHA；分支已变化时返回 409，用户需重新加载。编辑器提交使用 `skip_pipeline: true`，保留索引和事件但不自动启动流水线。未保存内容有离开确认和当前标签页的草稿恢复。

运行详情展示本次配置的只读任务图。日志按尝试和字节段加载，默认每段 64 KiB；用户可暂停跟随、搜索当前段、切换尝试及下载当前段。支持浏览器原生文件保存接口时，可逐段下载整份日志；不支持时明确保留分段下载能力，不将整份日志拼入内存。

以上界面行为需结合权限、失败状态和浏览器检查验收；构建通过不代表所有浏览器、数据库或分布式部署均已验证。

</div>
