<div lang="zh-CN">

[简体中文](oci.md) · [English](../../en/deployment/oci.md)

# OCI 注册表部署

公开 OCI API 由 Gity Rust 网关提供。Angos 是内部存储和分发引擎：单进程安装时绑定到 loopback，拆分安装时绑定到私有容器网络。不要公开 Angos 端口或 Web UI。上游镜像版本固定在 [`build/package/angos-engine.md`](../reference/angos-engine.md) 中。

网关会对每个请求执行平台的不透明令牌校验和项目 ACL 检查，包括由本地缓存响应的请求。Angos 监听器不是另一个公开认证服务。跨仓库挂载请求要求调用者对源仓库有拉取权限，对目标仓库有推送权限。网关代理请求前会移除客户端的 `Authorization` 请求头，因此上游注册表凭据不会从 Gity 客户端会话中获取。

TOML 示例展示了 Angos 文件系统和 SeaweedFS S3 设置，以及 pull-through 仓库条目。应由秘密管理提供方生成权限为 mode-0600 的文件来保存上游凭据和 S3 凭据；不要将凭据值写入 Gity 配置或仓库。Angos 支持合并多个配置文件，因此基础配置可以保存在部署源码管理中，再由秘密管理提供方挂载覆盖配置文件。Angos v1.12.4 的 S3 客户端使用 path-style 端点 URL，且没有提供 `force_path_style` 设置。

在 Gity 网关上，镜像模式为只读。其类型化配置必须列出注册表别名、允许注册表和令牌服务通信的精确 HTTPS authority，以及秘密引用。OCI 客户端传输层会针对主机允许列表检查每个请求和重定向，拒绝降级到 HTTP，并在跨源请求时移除 Authorization。客户端不能任意选择主机，也不能转发自身凭据。每次请求镜像清单前，Gity 都会解析已配置的上游引用，并将解析出的摘要发送给 Angos；可变版本标签也采用此流程。上游返回 401、403 或发生传输错误时，请求会在到达 Angos 前停止，因此缓存中的旧标签不会掩盖上游拒绝请求或服务中断。每次命中缓存时仍会执行本地项目 ACL 检查。解析器使用上游 OCI Distribution 客户端及 Gity 传输适配器，详见 [`build/package/oci-client.md`](../reference/oci-client.md)。

Gity 使用稳定项目 ID 派生存储键，因此修改用户可见的命名空间不会移动对象。平台授权适配器负责将公开仓库路径映射到稳定的 Angos 仓库键。

在集成环境中，通过 Angos 的标准 S3 接口使用 SeaweedFS。宣布支持这一组合前，必须针对确切固定的 Angos 版本验证推送和拉取、分段上传、Range 请求、缓存命中和未命中、可变标签刷新、离线行为，以及未经授权的缓存命中。在记录这些运行结果之前，不宣称 OCI 黑盒验收通过。

</div>
