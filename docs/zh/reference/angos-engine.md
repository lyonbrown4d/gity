<div lang="zh-CN">

[简体中文](angos-engine.md) · [English](../../en/reference/angos-engine.md)

# Angos 引擎资产

Gity 将上游预编译引擎作为内部子进程运行，不会复制或修改 Angos 应用源码。

| 字段 | 值 |
| --- | --- |
| 上游项目 | [project-angos/angos](https://github.com/project-angos/angos) |
| 版本 | `v1.12.4` |
| 源码提交 | `c4cad324360967746fabf3ebf9528601bede7ae6` |
| 许可证 | Apache-2.0 |
| 镜像来源 | `ghcr.io/project-angos/angos:1.12.4` |
| OCI 索引 SHA-256 | `1b90f6720c43eb143f519aec4c7bb721f180e74536634b64912085e464c65abd` |
| Linux amd64 清单 SHA-256 | `2d9f69b2f6ed652cae70406c2387235406c00c652f08feb7e37f9d8413b41584` |
| Linux arm64 清单 SHA-256 | `7192ea5a7ce9555a2f7c1806a3bd0c0b8921371b73626c9e7d075b26acc54dd3` |
| 上游镜像中的可执行文件 | `/angos` |
| 镜像修订标签 | `c4cad324360967746fabf3ebf9528601bede7ae6` |

`build/package/Dockerfile.variants` 通过索引摘要固定镜像，并且只复制 `/angos`。该可执行文件是静态 PIE，不依赖 ELF 解释器。在 Alpine 3.22 中运行 `--help` 成功，因此 Alpine 镜像会复用官方二进制文件，而不重新构建上游源码。OCI 和完整 standalone 目标可以包含该文件；其他变体不包含。上游版本提供 Cosign 签名；更新此固定版本时应验证签名。

</div>
