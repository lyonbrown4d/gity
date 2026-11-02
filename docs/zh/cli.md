<div lang="zh-CN">

[简体中文](cli.md) · [English](../en/cli.md)

# gity 命令行工具

`gity` 是 Gity 服务器的命令行客户端，命令结构和习惯对齐 GitLab 的 `glab` 与 GitHub 的 `gh`：一个 YAML 文件记录当前服务器和各服务器的令牌，`--host`、`--token`、`--json`、`--config` 是全局参数，放在任何子命令前后都可解析。二进制名为 `gity`，Cargo 包名为 `gity-cli`。

客户端只调用服务端 REST API，前缀固定为 `/api/v1`，自身不维护另一套接口定义。默认输出是对齐的表格或 `键: 值` 行，`--json` 打印接口返回的原始 JSON（带缩进）。表格没有结果时打印 `(no results)`，这不是错误。颜色只在标准输出是终端且未设置 `NO_COLOR` 时启用。

## 安装

在仓库根目录安装：

```bash
cargo install --path apps/gity-cli
gity --version
```

可执行文件安装到 Cargo 的 bin 目录（默认 `~/.cargo/bin/gity`）。`gity repo clone` 会调用本机的 `git`，因此需要 PATH 中存在 Git。

`gity completion` 输出 shell 补全脚本，参数取 `bash`、`elvish`、`fish`、`powershell` 或 `zsh`：

```bash
gity completion bash > /etc/bash_completion.d/gity
gity completion zsh > "${fpath[1]}/_gity"
gity completion fish > ~/.config/fish/completions/gity.fish
```

## 认证

两种登录方式都会把令牌写入配置文件。第一种直接使用已有的个人访问令牌（PAT）：

```bash
gity auth login --host http://127.0.0.1:8080 --token gity_xxx
```

第二种用账号密码换取一个新的 PAT：

```bash
gity auth login --host http://127.0.0.1:8080 --username admin --password 'change-me'
```

`--host` 可以省略，此时取全局 `--host` 或环境变量 `GITY_HOST`；`--token` 同样可来自 `GITY_TOKEN`。用户名密码方式不保存密码：客户端先向 `/api/v1/auth/login` 提交账号密码取得会话令牌，再用该会话在 `/api/v1/users/{user_id}/tokens` 创建一个名为 `--name`（默认 `gity-cli`）的 PAT，最终只保存这个 PAT。只给 `--username` 不给 `--password` 会报 `--password is required together with --username`；两者都没给且没有 `--token` 时报 `pass --token, or --username and --password`。

写入配置前，客户端用得到的令牌请求 `/api/v1/users/me` 验证凭据，然后把该服务器和令牌存为当前配置：

```bash
gity auth login --host http://127.0.0.1:8080 --token gity_xxx
# signed in as admin on http://127.0.0.1:8080
# credentials stored in ~/.config/gity-cli/config.yaml
```

`gity auth status` 请求 `/users/me` 后打印当前服务器、用户名和管理员标记：

```bash
gity auth status
# host:  http://127.0.0.1:8080
# user:  admin
# admin: false

gity auth status --json
# {"host": "http://127.0.0.1:8080", "user": { ... }}
```

`gity auth logout` 从 `hosts` 中删除当前服务器的条目，并把剩余的第一个 host 设为当前服务器。它只清理本地凭据，不会吊销服务端令牌；要让令牌失效请用 `gity token revoke`。没有存过凭据时打印 `no stored credentials`，命令仍然成功。

## 配置

配置是一个 YAML 文件，默认位于系统配置目录下的 `gity-cli/config.yaml`：Linux 为 `~/.config/gity-cli/config.yaml`，macOS 为 `~/Library/Application Support/gity-cli/config.yaml`。用 `--config <path>` 或 `GITY_CONFIG` 指定其他文件。写入时自动创建父目录，并在 Unix 上把文件权限设为 `0600`。

```yaml
host: http://127.0.0.1:8080
hosts:
  http://127.0.0.1:8080:
    token: gity_xxxxxxxxxxxxxxxx
    user: admin
  http://git.example.com:
    token: gity_yyyyyyyyyyyyyyyy
git_protocol: https
editor: null
```

`host` 是当前服务器；读写时都会去掉地址结尾的 `/`，`hosts` 的键必须与去掉结尾斜杠后的地址完全一致，否则取不到该服务器的令牌。`hosts.<host>.token` 是 PAT，`user` 是登录时记录的用户名。`git_protocol` 默认 `https`，只接受 `https` 或 `ssh`。`editor` 默认为 `null`。`gity repo clone` 使用服务端返回的 `clone_http_url`；`git_protocol` 是经过校验并保存的配置项，不改变克隆使用的协议。

`gity config` 系列命令读写这个文件：

| 命令 | 行为 |
| --- | --- |
| `gity config get <key>` | 打印 `host`、`git_protocol` 或 `editor` 的值，其他键名报错 |
| `gity config set <key> <value>` | 写入其中一个键并保存文件，随后打印 `key = value` |
| `gity config list` | 打印文件路径、当前 host、`git_protocol`、`editor` 和已配置的 hosts |
| `gity config host add <host> --token <token>` | 为该服务器保存令牌并设为当前服务器 |
| `gity config host remove <host>` | 删除该服务器条目，未配置时报错 |
| `gity config host list` | 以 `HOST`、`USER`、`AUTH`、`CURRENT` 表格列出已配置的服务器 |

`host add` 的 `--token` 也可由 `GITY_TOKEN` 提供。`host list` 用 `*` 标出当前服务器，`AUTH` 列在有令牌时显示 `token`。

环境变量与优先级：

| 变量 | 等价参数 |
| --- | --- |
| `GITY_HOST` | `--host` |
| `GITY_TOKEN` | `--token`（`auth login` 与 `config host add` 的 `--token` 同样读取） |
| `GITY_CONFIG` | `--config` |

优先级从高到低为命令行参数、环境变量、配置文件：环境变量作为参数的默认值，命令行给值时覆盖它。没有 `--host`/`GITY_HOST` 时取配置文件的 `host`，都没有则报 `no server configured: pass --host or run gity auth login --host <url>`；没有 `--token`/`GITY_TOKEN` 时只从当前 host 的 `hosts` 条目取令牌，仍然没有则报 `not signed in to <host>: pass --token or run gity auth login`。

## 命令参考

### 全局参数

| 参数 | 作用 |
| --- | --- |
| `--host <url>` | 服务器地址，等价 `GITY_HOST` |
| `--token <token>` | 个人访问令牌，等价 `GITY_TOKEN`，覆盖配置文件中的值 |
| `--json` | 打印接口返回的原始 JSON 而不是表格；只影响会输出接口数据的子命令，`gity api` 始终打印 JSON |
| `--config <path>` | 配置文件路径，等价 `GITY_CONFIG` |

### auth

| 命令 | 作用 |
| --- | --- |
| `gity auth login` | 保存凭据，参数为 `--host`、`--token`、`--username`、`--password`、`--name` |
| `gity auth status` | 显示当前服务器、用户、是否管理员 |
| `gity auth logout` | 删除本地保存的凭据 |

### config

参数与键名见[配置](#配置)一节。

### token

| 命令 | 作用 |
| --- | --- |
| `gity token list` | 列出当前用户的令牌 `ID`、`NAME`、`CREATED`，接口不返回令牌内容或哈希 |
| `gity token create --name <name>` | 创建令牌并打印一次明文，同时给出 `export GITY_TOKEN=...` |
| `gity token revoke <id>` | 按 ID 吊销令牌 |

### repo

| 命令 | 作用 |
| --- | --- |
| `gity repo clone <project> [directory]` | 按服务端返回的 `clone_http_url` 执行 `git clone`；`--with-token` 把令牌嵌入 URL |

`<project>` 可以是数字 ID，也可以是 `group/project` 路径或项目名，客户端会列出可见项目并按 `full_path`、`path`、`name` 匹配。省略 `directory` 时使用项目的 `path_key`，该字段为空时用 `project`。`--with-token` 生成 `http://oauth2:<token>@host/path` 形式的地址，令牌取自 `--token` 或 `GITY_TOKEN`，两者都没有时命令直接报错；长期使用的检出仓库不要带令牌，便于之后清理凭据。

### project

| 命令 | 作用 |
| --- | --- |
| `gity project list` | 以 `ID`、`PATH`、`VISIBILITY`、`NAME` 表格列出当前用户可见的项目 |
| `gity project view <project>` | 逐行打印 id、路径、名称、可见性、默认分支和 clone 地址 |

### issue

| 命令 | 作用 |
| --- | --- |
| `gity issue list <project> [--state <state>]` | 列出 issue；服务端列表接口不读取 `state` 参数，客户端按 `status` 自行过滤，`--state all` 返回全部 |
| `gity issue view <project> <iid>` | 打印标题、状态、标签、指派人、里程碑、描述和全部评论 |
| `gity issue create <project> --title <title> [--description <text>]` | 新建 issue |
| `gity issue close <project> <iid>` | 把状态改为 `closed` |
| `gity issue comment <project> <iid> --body <text>` | 添加评论 |
| `gity issue assign <project> <iid> <user-id>...` | 用给定用户 ID 替换指派人集合，至少给一个 |
| `gity issue unassign <project> <iid> <user-id>...` | 从当前指派人中移除给定用户 |
| `gity issue labels <project> <iid>` | 打印该 issue 的标签 |
| `gity issue label <project> <iid> --add <name> --remove <name>` | 按名称增删标签，至少给一个 `--add` 或 `--remove` |
| `gity issue milestone <project> <iid> --set <id>` 或 `--clear` | 设置或清空里程碑，二者只能给一个 |
| `gity issue milestones <project>` | 以 `IID`、`TITLE`、`STATE`、`DUE` 表格列出项目里程碑 |

### mr

| 命令 | 作用 |
| --- | --- |
| `gity mr list <project> [--state <state>]` | 列出 MR；服务端列表接口不读取 `state` 参数，客户端按 `state` 自行过滤，`--state all` 返回全部 |
| `gity mr view <project> <iid>` | 打印标题、状态、源分支与目标分支、描述，以及 `/checks` 返回的检查项 |
| `gity mr create <project> --title <title> --source <branch> --target <branch> [--description <text>]` | 创建 MR |
| `gity mr merge <project> <iid> [--squash]` | 合并 MR；`--squash` 把源分支提交压成一个 |
| `gity mr diff <project> <iid>` | 打印 diff，接口返回文本时原样输出，否则打印 JSON |

### pipeline

| 命令 | 作用 |
| --- | --- |
| `gity pipeline list <project>` | 以 `ID`、`STATUS`、`REF`、`SHA` 表格列出流水线 |
| `gity pipeline view <project> <pipeline>` | 打印流水线字段并附任务表（ID、名称、状态、阶段） |
| `gity pipeline jobs <project> <pipeline>` | 以 `ID`、`NAME`、`STATUS`、`STAGE` 表格列出作业 |
| `gity pipeline run <project> [--ref <ref>] [--sha <sha>]` | 触发流水线，请求体带 `source: cli`；不给 ref 时服务端使用项目默认分支，`--sha` 直接指定提交 |
| `gity pipeline retry <project> <pipeline>` | 用新的作业尝试重跑已结束的流水线 |
| `gity pipeline cancel <project> <pipeline>` | 取消运行中的流水线 |

### completion

| 命令 | 作用 |
| --- | --- |
| `gity completion <shell>` | 输出补全脚本，shell 取 `bash`、`elvish`、`fish`、`powershell`、`zsh` |

### api

| 命令 | 作用 |
| --- | --- |
| `gity api <path> [-X <method>] [-f key=value]...` | 直接调用 `/api/v1` 下的接口，打印响应中的 `data` |

`path` 以 `/` 开头或直接写 `projects` 都可；`-X` 取 `get`、`post`、`patch`、`put`、`delete`，默认 `get`，`put` 发送 PUT 请求；`-f` 可重复，值能解析为 JSON 时按 JSON 类型发送（`-f organization_id=1` 发送数字），否则按字符串发送，没有 `-f` 时发送空对象。响应始终打印为缩进 JSON，加 `--json` 也一样。

## 常见用法

以下示例针对本机服务器 `http://127.0.0.1:8080`。

登录并确认身份：

```bash
gity auth login --host http://127.0.0.1:8080 --username admin --password 'change-me'
gity auth status
gity config host list
```

先查组织，再创建项目。`gity` 没有内置创建项目的子命令，用 `gity api` 调用同一套 REST 接口：

```bash
gity api /orgs
gity api /projects -X post -f name=demo -f path=demo -f organization_id=1 -f visibility=private
gity project list
gity project view demo
```

`POST /projects` 需要 `name` 和 `organization_id`，可选 `path`（路径键，缺省取 `name`）、`default_branch`（缺省 `main`）、`visibility`（`private`、`internal` 或 `public`，缺省 `private`）和 `description`。

克隆仓库并开分支：

```bash
gity repo clone demo
cd demo
git switch -c feature/login
```

提交 issue 并跟进：

```bash
gity issue create demo --title "登录页返回 500" --description "复现步骤见附件"
gity issue list demo
gity issue view demo 1
gity issue comment demo 1 --body "已定位到会话解析"
gity issue label demo 1 --add bug --add P1
gity issue close demo 1
```

推送分支、开 MR 并合并：

```bash
git push -u origin feature/login
gity mr create demo --title "修复登录页 500" --source feature/login --target main
gity mr view demo 1
gity mr diff demo 1
gity mr merge demo 1 --squash
```

运行流水线要求仓库默认分支上存在 `.gity-ci.yml`：

```bash
gity pipeline run demo --ref main
gity pipeline list demo
gity pipeline jobs demo 1
gity pipeline retry demo 1
```

脚本里读取原始 JSON：

```bash
gity issue list demo --json | jq '.[].title'
gity api /users/me | jq .username
```

## 退出与错误

命令成功时退出码为 `0`。任何失败都由 `main` 返回错误，运行时把 `Error: <消息>` 打印到标准错误并以退出码 `1` 结束。服务端返回的失败形如 `<message> (HTTP <status>)`，例如 `insufficient project permissions (HTTP 403)`；响应中 `code` 不为 `0` 或状态码非成功都算失败。解析项目引用失败这类本地错误只打印消息，例如 `project 'demo' not found`。

`gity repo clone` 直接判断 `git clone` 的退出状态，失败时报 `git clone failed with <status>`；本机没有 `git` 时报 `run git clone; make sure git is installed`。

输出到管道时不带颜色，因此 `gity ... | grep`、`jq` 之类的处理是安全的。需要稳定的机器可读输出时用 `--json`，或直接 `gity api`，不要解析表格。空表格打印 `(no results)`，退出码仍为 `0`。

</div>
