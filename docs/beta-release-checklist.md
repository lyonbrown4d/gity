# Rust Beta 发布清单

`codex/rust-full-migration` 已完成 Rust 替换、遗留代码清理和 Linux x86_64 本机验收。该分支尚未合并或发布。推送后先通过 Linux x86 CI；发布工作流由 tag 触发，须在打正式发布 tag 前确认目标架构构建成功。

## 本机发布检查

使用固定 Rust 工具链、WASM target、Trunk 和 Docker 执行：

```powershell
.\scripts\release-check.ps1
.\scripts\blackbox-e2e.ps1
```

发布检查包括源码和第三方资产校验、格式、Clippy、workspace 测试、WASM 检查及 Rust/Trunk 发布构建。Docker smoke 检查 Rust server 镜像健康路由、OpenAPI 和 Web 根页面。清理后的 Linux 本机发布构建、workspace、严格 Clippy 及冻结源码 server/Runner 镜像复验均已通过。PowerShell 脚本本身未实测。

浏览器、SSH、S3 和 Runner 容器任务的实测范围见[迁移验证记录](rust-migration.md)。Release 支持 Release 创建、修改、删除；外部 asset link 支持创建、读取、删除，不支持更新；tag 支持创建、删除，不支持修改。

## GitHub Actions 发布流程

`.github/workflows/ci.yml` 在分支推送时运行 Linux x86 CI；它不验证 ARM64。`.github/workflows/release.yml` 由 tag 触发，构建 Linux amd64/arm64 归档及 server、runner、search-index 镜像，并发布校验和。ARM64 等其他架构的验证须以发布工作流的实际结果为准。

迁移分支尚未合并主分支或发布。主分支仍保留迁移前实现；不得把分支 push CI 说成 ARM64 发布通过。

## 发布门槛

- [x] 完成代表性产品流程和权限核对。
- [x] 完成原版代表页面视觉对照及 Release/Runners 浏览器功能验收。
- [x] 使用真实客户端验证 SSH clone、push、pull、密钥隔离和分支保护。
- [x] 使用 SeaweedFS 验证 S3 认证、对象字节、Range 和分段上传。
- [x] 清理旧 Go/React 实现及旧构建发布配置；清理后 workspace 为 149 passed、0 failed、1 ignored，Runner Docker 用例单独 1 passed。
- [x] 清理后 `cargo xtask release`、源码检查、vendor 检查、格式及 native/WASM 严格 Clippy 通过。
- [ ] 推送迁移分支，通过 Linux x86 CI；tag 触发的发布工作流通过后再合并或发布。
- [ ] 在真实环境验证完整生产备份与恢复流程。

压力测试已取消，不作为发布门槛；未执行吞吐量或性能曲线测试。Linux x86_64 以外平台和 PowerShell 发布脚本尚未实测。
