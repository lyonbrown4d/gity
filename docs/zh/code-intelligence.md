<div lang="zh-CN">

[简体中文](code-intelligence.md) · [English](../en/code-intelligence.md)

# 代码智能

网页代码阅读需要五类查询：文件大纲、跳到光标所在的声明、按名查声明、悬停文档、声明名搜索。这些查询由项目默认分支的符号快照回答，快照由 Runner 扫描、平台入库产生。

当前交付的是**词法层**：tree-sitter 逐文件抽取声明，不解析引用关系。所以“跳转到定义”实际是“跳转到光标所在的声明”——它能说明某一行属于哪个 `fn` 或 `struct`，不能说明某个标识符指向哪个声明。精确层（SCIP）的 shard 格式与查询实现已经在本仓库中，但还没有接进平台，见后面的“精确层（下一阶段，尚未上线）”。

## 能力与限制

| 能力 | 端点 | 当前实现的行为 |
| --- | --- | --- |
| 文件大纲 | `GET /api/v1/projects/{id}/repository/symbols` | 按 `path` 列出该文件抽出的全部声明，按行号、列号排序 |
| 跳到声明 | `GET /api/v1/projects/{id}/repository/definitions` | 返回覆盖光标位置的最内层声明，`mode` 固定为 `enclosing` |
| 按名查声明 | `GET /api/v1/projects/{id}/repository/references` | 全快照中名字完全相等的声明，不是引用点 |
| 悬停文档 | `GET /api/v1/projects/{id}/repository/hover` | 返回光标所在声明的名称与文档首行，`source` 固定为 `lexical` |
| 声明名搜索 | `GET /api/v1/projects/{id}/repository/code-search` | 对声明名与文件路径做全文检索 |

每个响应都带 `precision`（当前恒为 `approximate`）和 `index`（索引是否可用、提交、入库时间）。这两个字段按两层并存的结构设计，精确层上线后响应形状不变，只有取值变化。

限制需要直说：

- **只有词法层**。单个文件内的声明位置是准确的；跨文件的名字解析、真正的定义跳转、引用点统计都不存在。
- **宏不展开**。宏生成的声明在源码里没有对应语法节点，抽不到。
- **重载与同名不区分**。按名查询会返回所有同名声明，不区分参数表、trait 实现和所属模块，调用点无法判断指向哪一个。
- **动态分派、别名与 `use` 不跟随**。名字解析器尚未实现，`use` 语句只作为导入记录写进归档，不参与查询。
- **只支持 Rust**。`Language::from_path` 目前只认 `.rs`，其他语言的文件不会被抽取。
- **方法与字段**。`impl` 内的函数以 `kind=function` 出现，`scope` 是 impl 的类型名；字段与局部变量不抽取，`method`、`field`、`variable` 三种 kind 目前没有产出。
- **文档只取一行**。`///`、`//!` 注释块中的第一条非空行进入 `doc`，不是整段文档。
- **最内层是行号优先**。`definitions` 在覆盖光标的声明里取起始行最晚的一个；起始行相同时优先列号不大于光标的声明，好让 `Foo::bar` 落在 `bar` 而不是 `Foo`。

## 索引是怎么产生的

用户不需要写 `.gity-ci.yml`，也不需要声明流水线：索引任务是平台在 push 之后自己入队的系统任务。

### 平台自动入队

默认分支 push 成功、提交不是全零、且语言统计里出现 Rust 时，平台创建一个 `kind=index` 的任务。任务中固定的字段：

- `tags` 为 `["gity-indexer"]`，只有带该标签的 Runner 才领取；
- `artifact_name` 为 `code-index.msgpack.zst`；
- `pipeline_job_name` 为 `code-index`，Runner 侧看到的 `CI_JOB_NAME`；
- `trusted_config` 为 `false`，不使用仓库配置；
- 最大尝试次数为 2。

系统任务由服务端内部入口创建，公开的建任务接口只接受 `noop` 与 `script`，因此索引任务不会经过 YAML 校验，也不在流水线定义里；它在项目任务列表中以 `kind=index` 可见。标签 push、非默认分支 push、以及语言统计里没有 Rust 的仓库都不会入队。

### Runner 扫描并上传

领取任务后，Runner 在自己的进程内执行，全程不启动容器，也不需要镜像：下载固定提交的源码 ZIP（上限 256 MiB），按 100 000 个条目、512 MiB 解压总量解包，并拒绝 `.git` 元数据、越界路径和符号链接；用 tree-sitter 解析每个受支持的文件，生成 msgpack + zstd 归档（`FORMAT_VERSION = 1`）；归档超过 64 MiB 时直接报错，而不是截断上传；最后把归档作为任务产物上传。

租约失效、取消和 Runner 关闭都会终止扫描。

### 平台入库

Runner 提交成功状态后，平台对 `kind=index` 的任务执行入库：按名字找到产物、从存储读出字节、解压解码（格式版本不符即失败）、写入本地索引（归档、`current/` 与 `status.json`）。

入库失败**不会**改写任务状态：任务已经成功，失败只记录日志和 `gity_code_index_ingest_total{result="failure"}` 指标。也就是说“任务成功”不等于“索引已更新”，排查时要看这个指标和服务端日志。

## 索引任务的前置条件

索引任务只有带 `gity-indexer` 标签的 Runner 能领取。标签在注册 Runner 时写入，逗号分隔，匹配时大小写不敏感：

```bash
curl -X POST "$GITY/api/v1/projects/42/runners" \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"name":"indexer","tags":"linux,gity-indexer"}'
```

注册需要维护者（角色等级 ≥ 40）身份，响应里的令牌只返回一次。没有这样的 Runner 时，任务一直停在 `pending`，既不报错也不超时失败，索引因此不会产生。standalone 部署同样如此：它启动的 Runner 进程使用注册接口发放的令牌，不会自带标签。

执行脚本任务的 Runner 不必带这个标签；反过来，只带 `gity-indexer` 而缺少脚本任务所需标签的 Runner 只领索引任务。

## 产物与配额

| 限制 | 值 | 超出后的行为 |
| --- | --- | --- |
| 单个源文件 | 512 KiB | 跳过该文件，不报错 |
| 文件数量 | 5000 | 停止遍历，索引只含已扫描部分 |
| 源码总字节 | 64 MiB | 停止遍历，同上 |
| 归档产物 | 64 MiB | Runner 报错、任务失败；服务端产物接口同样拒绝 |
| 源码 ZIP 解包 | 100 000 条目 / 512 MiB | 解包失败、任务失败 |

扫描配额是硬编码的 `scan::Limits::default()`，不读 `search.max_files_per_project` 与 `search.max_file_size_bytes`，后两者属于文本搜索索引。

触到配额时**没有任何 `truncated` 标记**：扫描到达上限就安静停下，归档、`status.json` 和接口响应里都没有截断字段，索引看上去完全正常，只是少了文件。判断是否被截断只能比较 `status.json` 的 `files` 与仓库中的实际文件数。真正会明确报错的只有归档 64 MiB 和服务端产物上限。

扫描还会跳过 `.git`、`target`、`node_modules`、`vendor`、`dist`、`build`、`.venv`、`__pycache__`、`third_party` 目录，以及符号链接、非 UTF-8 文件和解析失败的文件。

## 存储布局

```text
data/code-index/                   search.code_index_root
├── 42.lock                        每个项目一把锁，读共享、写独占
└── 42/
    ├── current/                   tantivy 索引，meta.json 是存在标志
    ├── archive/
    │   └── <commit>.msgpack.zst   归档，msgpack + zstd，保留最近 10 份
    └── status.json                commit、files、definitions、archive_bytes、indexed_at
```

`search.code_index_root` 默认 `data/code-index`，可用 `GITY_SEARCH__CODE_INDEX_ROOT` 覆盖，启动时必须非空。根目录由服务启动时创建并规范化；项目路径是符号链接时拒绝访问。

锁与生命周期和文本搜索索引一致：`<root>/<project>.lock` 上的 `try_lock` 由 `fs2` 提供，读请求取共享锁、重建取独占锁，都不等待。差别在锁被占用时：代码索引的读请求直接报错，对外是 500；文本搜索索引则返回空结果并回退到 Git 扫描。重建期间出现的 500 重试即可。

一次重建先写归档，再整体删除并重建 `current/`，最后写 `status.json`，随后按修改时间把归档裁到 10 份。这里没有增量更新，`current/` 每次都是新的。

归档还会复制到对象存储的 `code-index/<project_id>/<commit>.msgpack.zst`，由 `search.code_index_remote`（默认开启）控制。这份副本供本地快照缺失时恢复使用；写入和读取失败只记日志与 `gity_code_index_remote_total` 指标，不影响索引本身。查询端点只读本地 store，本地缺失时不会触发远端取回。

## HTTP 端点参考

五个查询端点都是 `GET /api/v1/projects/{id}/...`，权限是项目读权限：公开项目的匿名请求可用，内部项目需要登录，私有项目需要项目成员身份。

### 端点与参数

| 端点 | 参数 | 响应字段 |
| --- | --- | --- |
| `/repository/symbols` | `path`（必填，仓库相对路径） | `path`、`items[]`、`precision`、`index` |
| `/repository/definitions` | `path`、`line`、`column`（必填，1 起始的正整数） | `definitions[]`、`mode`、`precision`、`index` |
| `/repository/references` | `name`（必填）、`limit`（1–500，默认 100） | `references[]`、`total`、`precision`、`index` |
| `/repository/hover` | `path`、`line`、`column` | `hover`、`precision`、`index` |
| `/repository/code-search` | `q`（或 `query`，必填）、`limit`（1–500，默认 50） | `matches[]`、`total`、`precision`、`index` |

`items[]`、`definitions[]`、`matches[]` 的元素形状相同：`path`、`name`、`kind`、`scope`、`line`、`line_end`、`column`、`doc`。`references[]` 的元素是 `path`、`name`、`line`、`column`、`scope`，没有 `doc` 与 `kind`。`index` 是 `available`、`commit`、`indexed_at`。

单文件大纲最多返回 4096 条声明，按名查询最多 512 条，再由 `limit` 截断。`references` 的 `total` 是 `limit` 截断前的条数（因此最大 512）；`code-search` 的 `total` 等于本次返回的条数。

### 响应外壳与降级

成功响应统一为 `{"code":0,"message":"OK","data":{...}}`。索引缺失不是错误：返回空数组、`index.available=false`、`precision="approximate"`，HTTP 状态仍是 200。

```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "definitions": [],
    "mode": "enclosing",
    "precision": "approximate",
    "index": {"available": false, "commit": "", "indexed_at": ""}
  }
}
```

索引可用时的同类响应：

```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "definitions": [
      {
        "path": "crates/gity-code-intel/src/store.rs",
        "name": "refresh",
        "kind": "function",
        "scope": "Store",
        "line": 77,
        "line_end": 99,
        "column": 3,
        "doc": "Indexes an in-memory archive, replacing the previous snapshot."
      }
    ],
    "mode": "enclosing",
    "precision": "approximate",
    "index": {"available": true, "commit": "5f1c0e2", "indexed_at": "2026-11-02T16:04:11.284Z"}
  }
}
```

`hover.hover` 为 `null` 表示光标位置没有声明；有结果时 `contents_markdown` 是文档加名称，没写文档时退化为 `` `name` (kind) ``，`source` 恒为 `lexical`。

### 错误码

| 状态码 | 触发条件 |
| --- | --- |
| 400 | `path`、`name`、`q` 缺失或为空；`line`、`column` 缺失或不是正整数；`limit` 不在 1–500 |
| 401 | 需要身份但没有有效凭据，例如私有项目的匿名请求 |
| 403 | 有身份但读不到该项目；项目令牌 scope 不足 |
| 404 | 项目不存在或已被删除 |
| 500 | 索引读取失败：重建持有独占锁、tantivy 报错、查询串无法解析 |

5xx 响应的 `message` 统一替换为 `internal server error`，具体原因只写服务端日志。

## 手动刷新

`POST /api/v1/projects/{id}/search/code-index/refresh` 立即从默认分支 HEAD 重建索引，不需要 Runner，也不必等下一次 push。

权限是 `RepositoryAdmin`：角色等级 ≥ 40（维护者或所有者），超级管理员同样通过。**项目令牌不能调用**，因为这个操作没有可授予项目令牌的 scope，用项目令牌会得到 403；要用用户令牌或会话。请求没有请求体，响应：

```json
{
  "code": 0,
  "message": "OK",
  "data": {
    "commit": "5f1c0e2",
    "files": 214,
    "definitions": 3621,
    "indexed_at": "2026-11-02T16:04:11.284Z"
  }
}
```

空仓库或默认分支不存在时返回 404（底层是 `git reference not found` 或 `git repository is empty`）；索引写入失败返回 500。本地重建失败时，服务会尝试用对象存储中同一提交的归档恢复，仍然失败才把错误返回。

## 精确层（下一阶段，尚未上线）

下面的内容是设计，代码位于 `crates/gity-scip`，平台目前没有任何调用方，端点也还不返回精确结果。

### SCIP shard 布局

Runner 侧运行语言索引器（`rust-analyzer scip`、`scip-typescript` 等），把 `index.scip` 以 zstd 压缩后按提交存放。服务端解压、解码 protobuf（解压后上限 512 MiB），再在本地生成 shard：

```text
<code-intel root>/<project>/scip/
├── manifest.json   commit、索引器、格式版本、列编码、计数
├── paths.bin       排序后的路径及其 occurrence 区间
├── occ.bin         occurrence 定长记录，按路径与位置排序
├── symbols.bin     仓库内符号：kind、文档与签名偏移
├── external.bin    依赖中定义的符号
├── texts.bin       索引器附带源码时的原文
├── docs.bin        文档与签名文本池
├── refs.bin        CSR：符号 → occurrence
├── coverage.bin    精确分析过的路径位图
└── lock            读写锁
```

`FORMAT_VERSION = 2`，版本 1 的 shard 直接拒绝，从保存的 `index.scip` 重建即可。manifest 最后写入，是提交点；`Shard::open` 把精确层当作可选，任何不可用情形都返回 `Ok(None)`。打开 shard 用 `memmap2` 映射八个表文件，一个句柄占八个文件描述符，应缓存句柄而不是每次请求重开。

### Answer 三态

| 取值 | 含义 | 调用方行为 |
| --- | --- | --- |
| `Found` | 找到定义位置；定义在依赖包中时 `external=true`，位置是使用点 | 跳转；`external` 时改为展示悬停文档 |
| `Absent` | 该文件被索引器精确分析过，但光标下没有符号 | 直接返回空，不再回退 |
| `Uncovered` | 文件不在精确分析范围内，或 shard 不可用 | 回退词法层 |

位置是 1 起始的行列，列单位按文档的 `PositionEncoding` 解释；`Shard::normalized_position` 把 UTF-8 列换算成该文档使用的单位。

### 回退承诺

精确层接入后，查询端点的响应结构不变：词法层继续回答未覆盖的文件，`precision` 从 `approximate` 变为精确值，`hover.source` 从 `lexical` 变为 `scip`，`definitions.mode` 从 `enclosing` 变为真正的定义解析。shard 缺失、损坏、被截断、版本不符或被写者锁住时，`Shard::open` 静默降级，请求不会因此失败——这是精确层能够不改变客户端就逐步铺开的前提。

## 故障排查

| 现象 | 原因 | 处理 |
| --- | --- | --- |
| 端点返回空数组，`index.available=false` | 项目还没有索引 | 确认索引任务是否入队；注册带 `gity-indexer` 标签的 Runner，或直接手动刷新 |
| 索引任务长期 `pending` | 没有带 `gity-indexer` 标签的 Runner | 注册 Runner 或补上该标签，并确认进程在运行 |
| 只有部分文件出现在大纲里 | 语言不支持、文件超过 512 KiB、位于跳过目录、非 UTF-8，或触到 5000 文件 / 64 MiB 上限 | 对照配额表；配额截断没有标记，只能比较 `files` 与仓库实际文件数 |
| 任务失败，报 `code index archive is N bytes ... limited to 64 MiB` | 归档超过产物上限 | 缩小索引范围；当前没有调高上限的配置项，也没有截断上传 |
| 任务成功但索引仍是旧提交 | 入库失败：产物缺失、解码失败、版本不符、存储不可读 | 查 `gity_code_index_ingest_total{result="failure"}`、`gity_code_index_enqueue_total` 与服务端日志，修好后手动刷新 |
| 查询或刷新返回 500 `internal server error` | 索引目录正被重建独占，或 tantivy / IO 报错 | 稍后重试；持续失败时检查 `search.code_index_root` 的权限与磁盘 |

</div>
