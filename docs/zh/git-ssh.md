<div lang="zh-CN">

[简体中文](git-ssh.md) · [English](../en/git-ssh.md)

# Git SSH 服务

SSH 使用 russh 在独立 TCP 端口提供 Git 传输，用户名固定为 `git`。服务不依赖系统 sshd 或系统用户账户，仅允许 `git-upload-pack` 与 `git-receive-pack`，不提供 shell、SFTP 或端口转发。

## 启用与主机身份

服务端配置使用 TOML。把下面配置合并到实际配置文件，完成独立数据库迁移后可用 `gity-server --config /path/to/gity.toml serve` 启动。环境变量使用 `GITY_SSH__` 前缀，例如 `GITY_SSH__ADDRESS=0.0.0.0:2228`。

```toml
[ssh]
enabled = true
address = "0.0.0.0:2222"
host_key = "/var/lib/gity/ssh_host_ed25519"
max_connections = 64
operation_timeout_seconds = 900
```

代码默认关闭 SSH；仓库提供的 standalone 和分布式 Git server Compose 配置已启用监听，使用 `${GITY_SSH_PORT:-2222}` 发布宿主机端口。需要按部署环境设置防火墙和入口范围。HTTP 端口不承载 SSH。

首次启用会创建 Ed25519 主机密钥；已有密钥会继续使用。Unix 上密钥必须是普通文件，且组和其他用户没有访问权限，例如 `0600`。Compose 中该文件位于 standalone 的 `gity-data` 卷或分布式的 `git-server-data` 卷。恢复时必须保留它；客户端通过受信渠道核对主机指纹并固定在 `known_hosts` 中，不能用重新接受未知密钥代替恢复验证。

## 个人 SSH 公钥

在账户页面的 SSH 公钥区域注册公钥，也可使用以下本人接口：

| 方法与路径 | 行为 |
| --- | --- |
| `GET /api/v1/users/me/ssh-keys` | 列出本人公钥、指纹、创建时间和最近使用时间。 |
| `POST /api/v1/users/me/ssh-keys` | 注册本人公钥，提交 `title` 和 `public_key`。 |
| `DELETE /api/v1/users/me/ssh-keys/{id}` | 撤销本人公钥；不存在或不属于本人时返回未找到。 |

```json
{"title":"work-laptop","public_key":"ssh-ed25519 AAAA..."}
```

接口以已认证账户确定归属，不能提交另一个用户 ID；管理员权限也不提供跨账户公钥管理入口。私钥始终保留在客户端，服务端不接收私钥。

个人公钥代表账户身份。每条 Git 命令都重新检查公钥是否存在、账户是否启用、项目是否有效，以及当前项目角色和可见性。已有 SSH 连接上的新 channel 也重新检查。私有项目需要访问权限；推送至少需要 developer 权限，保护分支继续按该账户的实际角色判断。撤销密钥、禁用账户或降低成员权限会影响后续命令，不会取消已经授权并开始的传输。

## 项目部署密钥

项目管理员通过 `POST /api/v1/projects/{id}/deploy-keys` 注册项目凭据：

```json
{"title":"build-host","public_key":"ssh-ed25519 AAAA...","can_push":false}
```

只读部署密钥允许 clone、fetch、pull；开启 `can_push` 后允许向所属项目写入。部署密钥不继承创建者的管理员或 owner 身份，保护分支按 developer 权限判断。每次执行 Git 命令重新检查项目、密钥和创建者状态；删除密钥或禁用创建者后，后续命令会被拒绝。部署密钥不因此获得其他项目访问权限。

两种公钥都通过 russh 解析并规范化，使用标准 SHA-256 指纹识别；改变备注不会形成新身份。服务拒绝私钥、`authorized_keys` 选项和非法公钥。个人公钥指纹不能重复注册，也不能与部署密钥共用；若并发注册形成跨模式歧义，SSH 会拒绝认证。

## 客户端与保护规则

先通过受信渠道准备主机公钥记录，然后配置客户端选择私钥并严格校验主机身份：

```sshconfig
Host gity.example
  User git
  Port 2222
  IdentityFile ~/.ssh/id_ed25519
  IdentitiesOnly yes
  StrictHostKeyChecking yes
  UserKnownHostsFile ~/.ssh/gity_known_hosts
```

```sh
git clone ssh://git@gity.example:2222/team/demo.git
cd demo
git push origin main
git pull --ff-only origin main
```

替换域名、端口和项目路径。保护规则覆盖已有和新建分支、通配规则、强推及删除，默认分支禁止删除。客户端环境不能覆盖服务端 hook 策略。`refs/gity/**` 为服务端合并回执保留，客户端更新会被拒绝，HTTP/SSH 广告也隐藏这些引用；相关机制见[合并恢复](merge-recovery.md)。

## 公共 revision 与内部回执

公共 REST 的仓库读取、归档和新建分支或标签所用的源 revision，只能解析到公开 `refs/heads/*` 或 `refs/tags/*` 可达的提交。公开历史中的完整 SHA、缩写 SHA 和 revision 表达式仍可使用；项目访问权限检查继续生效。

直接指定 `refs/gity/**` 会被拒绝。只由内部回执保留的提交和尚未应用的 prepared 候选，即使调用者知道完整 SHA、缩写或可解析到它的 revision 表达式，也不能通过公共接口读取或用作分支、标签的起点。内部回执读取及完整备份、复制仍可显式访问这些对象；隐藏广告不能代替这一可达性检查。

## 资源限制与验证

传输按流转发，不把仓库 pack 完整缓存在内存中。连接数、公钥认证 15 秒总等待和单次操作时间受限；断开、取消或超时会回收 Git 子进程。日志及 `gity_ssh_operations_total`、`gity_ssh_operation_seconds` 记录结果，不记录私钥或公钥认证签名。

相关源码测试命令：

```sh
cargo test -p gity-platform --lib ssh --locked
cargo test -p gity-git --test ssh --locked
```

这些测试与真实镜像黑盒的覆盖范围不同。Standalone/分布式 × Debian/Alpine 的实际 SSH、权限隔离及恢复验证结果以[验证状态](verification.md)中的运行证据为准。

此可达性约束适用于 REST 与应用侧 revision 解析。原生 Git 的保护按引用名执行：已知对象 SHA 的写入者可能把该对象引用到获准写入的普通分支。因此，隐藏引用不是同一仓库内的对象保密边界，回执中不能存放秘密。

</div>
