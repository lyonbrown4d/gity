<div lang="zh-CN">

[简体中文](release-checklist.md) · [English](../en/release-checklist.md)

# 发布检查

按实际交付范围选择检查，并在[验证状态](verification.md)记录结果与未覆盖项。

## 源码与构建

```sh
pnpm --dir web install --frozen-lockfile
pnpm --dir web build
cargo xtask source-check
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo xtask docs
```

确认 `apps/` 入口、组件 module、数据库版本、配置默认值和部署文件一致。镜像构建命令见[镜像变体](deployment/images.md)。

## 运行与交付

按范围核对[功能检查](functional-checks.md)、协议权限隔离、错误和恢复行为；两个部署模式和两个基础系统的结果分别记录。保留真实镜像 ID、平台、配置范围和证据，不从构建成功推断功能通过。

主分支应包含当前交付提交。部署前准备当前数据库、Git 仓库和对象存储备份；恢复能力以实际演练结果为准。文档构建产物与应用镜像分别发布，不把生成站点误当作服务运行验证。

</div>
