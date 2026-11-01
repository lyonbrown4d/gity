# HTTP 迁移基准

`legacy-http-routes.tsv` 在删除 Go 实现前，从 `internal/interfaces/http` 的非测试 `httpapi.Get/Post/Put/Patch/Delete` 声明提取，记录 234 组方法与路径。

系统路由使用 `/api` 前缀，业务路由使用 `/api/v1`。文件是契约数据，不包含旧实现。OpenAPI 回归按方法及路径形状核对覆盖，允许参数变量重命名；该检查不代替请求体、响应、权限或业务测试。

新增 Rust 接口无需改此基准。确实移除或更改兼容接口时，应明确评审影响后再修改对应记录，不能为通过检查而静默删除条目。
