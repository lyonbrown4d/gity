# Gity 产品路线与验收状态

## 产品方向

Gity 面向小团队和自托管场景，提供 Git 托管与协作能力。产品目标是让用户通过 Web 页面和常规 Git 工具完成主要工作流，而不是只提供 API。

当前交付范围为 Web：服务端使用 Rust、Axum、Tokio、SeaORM，Web 使用 Rust、Leptos 和 WebAssembly；FluxDI 负责服务端组合装配。Runner、Git hook 和搜索工具也是 Rust 应用。当前不交付桌面端。

CI 配置使用带类型校验的 YAML，仓库内默认文件为 `.gity-ci.yml`。数据库以当前 Rust 版本的版本化迁移为准；迁移程序只接受空数据库或迁移历史和校验值均匹配的 Rust 数据库，不接管旧 Go 数据库，也不转换历史 schema。

## 产品工作流目标

以下是产品核心工作流。Rust 替换已完成，代表页面的视觉与功能核验已完成；各流程的已验证范围和协议边界见迁移记录：

1. **仓库起步**：创建或导入项目、配置凭据、克隆、推送，并浏览 README、代码、分支和提交。
2. **协作**：创建 Issue、分配和标记工作、创建分支和 Merge Request、评审审批、合并并查看审计记录。
3. **CI**：编辑并校验 YAML、创建流水线、由项目范围内 Runner 执行、查看日志和产物，并取消或重试任务。
4. **包与发布**：发布并查看包、版本、Git 标签、Release 和关联资产。
5. **运维**：管理用户、项目、Runner、存储和审计，并验证升级与数据恢复流程。

## 当前实现边界

- 权限由服务端执行。项目角色为 `guest`、`reporter`、`developer`、`maintainer`、`owner`；项目有效角色取直接项目成员与所属组织成员角色的较高者。项目令牌按自身 scope 授权，不能借用创建者权限。
- Git 裸仓库的物理存储名由不可变项目 ID 生成，形如 `{project_id}.git`。组织或项目路径可变，改名会更新 URL 使用的 `full_path`，不移动或重命名 Git 物理仓库。
- 数据库支持 SQLite、PostgreSQL 和 MySQL 的当前版本迁移。新安装从空库初始化，已有 Rust 数据库须通过迁移历史及校验值验证。
- Runner 执行来自项目配置的脚本，默认应按可信项目和可信 Runner 主机使用。密钥脱敏是尽力处理，不构成防止秘密泄露的安全边界。
- 包注册表协议覆盖仍有限；CODEOWNERS 和审批规则并非 GitLab 全量语义。代码搜索索引和增量更新能力仍有边界。
- 高可用、多节点部署、灾难恢复与备份恢复工具不是当前已验收能力；SSH Git、Runner 隔离也应以对应实现和验证记录为准，不能从产品目标推断完成。

## 发布验收状态

Rust 代码替换、遗留代码清理和 Linux x86_64 本机验收已完成。`codex/rust-full-migration` 尚未合并主分支或发布。

截至 2026-11-01，本机已验证 PostgreSQL 18 与 SeaweedFS 4.51 集成环境、真实 Git/SSH 双克隆操作、Runner 容器任务及产物链路；Rust Releases/Runners 页面和原版代表页面视觉对照已完成。各项范围、最终复跑状态和限制见[Rust 迁移与验证记录](docs/rust-migration.md)。

发布前仍需完成：

- Linux x86_64 本机和冻结源码 server/Runner 镜像复验均已通过；正式发布前仍须由 tag 触发的 release workflow 验证发布架构。
- 推送迁移分支后通过 Linux x86 GitHub CI；Linux arm64 等平台由 tag 触发的 release workflow 构建验证，不能以分支 push CI 代替。
- 完成完整生产备份与恢复流程验证。其他架构及 PowerShell 发布脚本尚未实测。

压力测试已取消，不作为迁移或发布门槛；没有吞吐量或性能曲线实测结果。

验证记录应区分源码检查、编译、已运行测试和浏览器/部署验收；不得把某一项成功写成整体发布就绪。
