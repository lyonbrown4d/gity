<div lang="zh-CN">

[简体中文](functional-checks.md) · [English](../en/functional-checks.md)

# 功能检查

先按[Compose 操作](deployment/compose.md)准备隔离环境。以下是当前功能检查范围，不表示这些用例已全部执行通过。

1. 账户与权限：注册、登录、错误密码、令牌 scope、撤销和私有项目隔离。
2. Git：创建项目与文件提交，使用真实 HTTP/SSH 客户端 clone、push、pull，核对密钥隔离和分支规则。
3. 协作：Issue、成员、MR 评审与合并；核对权限、错误和审计行为。
4. CI：提交 `.gity-ci.yml`，注册项目 Runner，领取和执行任务，核对源码、日志、产物、取消和重试。
5. 软件包：使用原生客户端发布、下载和消费包，检查 Generic Range、校验和、幂等与跨项目隔离。
6. OCI：真实 push/pull，令牌 scope、撤销和跨仓库 mount 权限；mirror 拉取、缓存内容一致性与只读策略。
7. 分布式：组件停止与恢复，Git 副本代次、拒绝旧写入和受控提升；核对其他组件的可用边界。
8. Web：当前 React/Vite 的 JavaScript、CSS 与本地编辑器资源、语言切换、关键页面及交互。

实际镜像工具为 `cargo xtask blackbox`，参数与四种组合见[镜像说明](deployment/images.md#镜像黑盒验收)。默认删除本次专用项目的数据卷；不要使用保留环境的项目名。完整运行状态见[验证记录](verification.md)。

</div>
