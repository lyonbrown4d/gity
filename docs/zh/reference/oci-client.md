<div lang="zh-CN">

[简体中文](oci-client.md) · [English](../../en/reference/oci-client.md)

# OCI 客户端依赖

镜像清单解析器使用上游 ORAS Rust OCI 客户端，并通过轻量 HTTP 传输适配器执行出站主机和 HTTPS 策略。Gity 没有 fork 或复制客户端实现代码。

| 字段 | 值 |
| --- | --- |
| 上游项目 | [oras-project/rust-oci-client](https://github.com/oras-project/rust-oci-client) |
| Crate | `oci-client` |
| 已发布版本 | `0.18.0` |
| 固定的源码状态 | 提交 `a913569b1918ea1fda72ed3ba3711b93dea9100f` 对应的 `0.19.0` 开发版本 |
| 许可证 | Apache-2.0 |
| 固定方式 | Cargo git 依赖使用精确的 `rev`；Cargo.lock 记录相同提交 |

使用该精确提交，是因为其中受支持的 `new_with_transport` API 可让 Gity 校验每个注册表、Bearer 令牌 realm 和重定向目标。crates.io 上的 `0.18.0` 版本没有提供此传输 API。实现此集成时，固定提交对应的源码尚未作为 crates.io 版本发布。

</div>
