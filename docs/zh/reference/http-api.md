<div lang="zh-CN">

[简体中文](http-api.md) · [English](../../en/reference/http-api.md)

# HTTP API 契约

服务端通过 Utoipa 宏从实际类型与路由生成契约。只有显式启用 `api-docs` 的开发构建提供 `/docs` 和 `/openapi.json`；release 不提供文档端点或 Swagger 资源。GraphQL、MCP、配置及边界参见[多协议 API](api-transports.md)。

项目 API 请求必须携带对应用户或项目令牌，权限和 scope 在服务端执行。对外 JSON 的实体 ID 使用字符串。具体调用示例见[API 与权限示例](../api-examples.md)。

`contracts/` 中的 TSV 文件用于工程检查数据，不是公开 API 文档的第二份权威正文。修改路由和请求响应时同步代码中的文档宏，并核对生成结果。

</div>
