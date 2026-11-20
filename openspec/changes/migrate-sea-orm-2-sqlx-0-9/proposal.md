# Proposal

## Why

FerrisKey 仍在 SeaORM 1.1 / SeaQuery 0.32 / SQLx 0.8 上，仓库已通过 Dependabot 忽略 SeaORM 2 和 SQLx 0.9 来避免依赖分裂。SeaORM 2.0.4 依赖 SQLx 0.9，单独升级 SeaORM 会引入两个互不兼容的 SQLx 类型体系；两者需要作为一个原子变更一起迁移。

## What Changes

- **BREAKING** 将 `ferriskey-core` 和 `ferriskey-api` 使用的 SeaORM 升级到 `2.0.4`。
- **BREAKING** 将 workspace 直接使用的 SQLx 升级到 `0.9.0`，包括连接池、监听器、原生查询、迁移和测试夹具。
- 适配 SeaQuery 1.0 与 SeaORM 2 的编译期 API 变化，包括 `ExprTrait`、原生 SQL executor 的 `*_raw` 方法、条件与连接类型。
- 保持现有 entity、relation 和数据库结构不变，不迁移到 SeaORM 2 的新 entity 格式，也不引入 nested ActiveModel 等新能力。
- 更新 Dockerfile 中的 `sqlx-cli` 到 `0.9.0`，并调整 SQLx 0.9 不再提供锁文件所带来的安装参数。
- 重新生成并验证 `Cargo.lock`，确保依赖图中不再混用 SQLx 0.8 与 0.9。
- 升级完成后移除 Dependabot 对 SeaORM 2 / SQLx 0.9 的临时忽略。

## Capabilities

### New Capabilities

无。此次变更只升级依赖与编译适配，不引入新的持久化系统行为。

### Modified Capabilities

无。现有数据库契约、业务流程和 API 行为保持不变；因此该变更将使用 `skip_specs: true`，不创建行为规范增量。

## Impact

- 依赖与构建：`core/Cargo.toml`、`api/Cargo.toml`、`Cargo.lock`、`Dockerfile`
- 核心代码：SeaORM entity、repository、raw SQL、连接池、PostgreSQL listener 和测试夹具
- 迁移工具：`sqlx-cli` 0.9、容器内迁移命令与本地开发流程
- 自动化：Dependabot 配置和 workspace CI 验证
