<div lang="zh-CN">

[简体中文](git-ssh.md) · [English](../en/git-ssh.md)

# Git SSH 服务

SSH 使用 [russh](https://docs.rs/russh/0.65.0/russh/server/) 在独立 TCP 端口提供 Git 传输。HTTP 端口不承载 SSH；SSH 不依赖系统 sshd 或系统用户账户。

## 启用与配置

把下列配置合并到服务端 YAML，随后启动 `gity-server serve`。环境变量使用 `GITY_SSH__` 前缀，例如 `GITY_SSH__ADDRESS=0.0.0.0:2228`。

```yaml
ssh:
  enabled: true
  address: 0.0.0.0:2222
  host_key: data/ssh_host_ed25519
  max_connections: 64
  operation_timeout_seconds: 900
```

首次启用会创建 Ed25519 主机密钥；已有密钥会继续使用。Unix 上密钥文件必须为普通文件且仅所有者可读写，例如权限 `0600`。部署时持久化该文件，客户端应通过受信渠道核对主机指纹并写入 `known_hosts`。

生产 Compose 对应 `GITY_SSH_ENABLED`、`GITY_SSH_BIND` 和 `GITY_SSH_PORT`，默认关闭监听、端口映射只绑定本机。需要外部访问时明确设置绑定地址及网络入口；主机密钥位于 `app_data` 卷。

## 密钥与权限

目前通过项目部署密钥认证，SSH 用户名固定为 `git`。项目管理员通过 `POST /api/v1/projects/{id}/deploy-keys` 注册真实 OpenSSH 公钥：

```json
{"title":"build-host","public_key":"ssh-ed25519 AAAA...","can_push":false}
```

服务端使用第三方库解析公钥，保存规范公钥和标准 SHA-256 指纹；备注变化不产生不同的密钥身份。不接受只有合法 Base64、实际内容却不是 SSH 公钥的数据。

- 只读部署密钥允许 clone、fetch、pull；开启 `can_push` 后允许向所属项目写入。
- 密钥不会继承创建者的超级管理员身份，保护分支按 developer 权限判断。
- 每次执行 Git 命令重新核对项目、密钥及创建者状态；删除密钥或禁用创建者后，后续命令拒绝访问。
- 保护规则同时覆盖已有和新建分支、通配规则、强推及删除；默认分支禁止删除。
- SSH 只允许 `git-upload-pack` 与 `git-receive-pack`。不提供 shell、SFTP 或端口转发，客户端环境不能覆盖服务端 hook 策略。

项目部署密钥是项目凭据；当前没有个人 SSH 密钥管理接口。私钥始终保留在客户端。

## 客户端操作

```sh
git clone ssh://git@gity.example:2222/team/demo.git
cd demo
git push origin main
git pull --ff-only origin main
```

使用 SSH 客户端配置或 `GIT_SSH_COMMAND` 选择私钥。示例中的域名、端口和项目路径需要替换为实际配置。

## 验证

```sh
cargo test -p gity-server --test git_ssh --locked -- --test-threads=1
cargo test -p gity-git --locked
```

集成测试使用临时目录、本机随机独立端口和严格主机密钥验证，运行真实系统 `git`、`ssh` 客户端。测试通过 API 注册密钥，覆盖两个独立克隆之间 push/pull 后的字节与提交一致性、分支与标签、只读/撤销/跨项目密钥、禁用创建者、保护分支、shell/SFTP 拒绝，以及关闭单个 channel 或服务时的子进程回收。

传输按流转发，不把仓库 pack 完整缓存在内存中。连接数、15 秒公钥认证总等待和单次操作时间受限；断开、取消或超时会回收 Git 子进程。日志及 `gity_ssh_operations_total`、`gity_ssh_operation_seconds` 指标记录结果，不记录私钥或公钥认证签名。

</div>
