# Design

## Context

当前 workspace 在 `ferriskey-core` 与 `ferriskey-api` 中同时直接依赖 SeaORM 1.1 和 SQLx 0.8，并让 SeaORM 连接池暴露的 `PgPool` 直接流入 `PgListener`、Compass 批量写入和大量 PostgreSQL 测试夹具。SeaORM 2.0.4 改用 SeaQuery 1.0 与 SQLx 0.9，因此只升级 SeaORM 会形成两个 SQLx 类型体系。

已确认的现状约束：

- `core/src/entity/` 约有 73 个 entity 文件，当前使用传统 `Relation` / `Related` 形式。
- 至少 18 处 SeaORM `execute` / `query_one` / `query_all` 接收 `Statement`，需要迁移到 `*_raw` API。
- SQLx 直接调用主要位于测试代码，但 `token_state/listener.rs` 与 Compass 批量写入在生产路径中直接使用 `PgPool`。
- `ConnectOptions::map_sqlx_postgres_opts`、`get_postgres_connection_pool()` 和 `PgListener` 都依赖 SeaORM 与 SQLx 的类型一致性。
- Dockerfile 当前安装 `sqlx-cli 0.8.6 --locked`；SQLx 0.9 不再在 crate 中提供 Cargo.lock。
- 当前 Rust 工具链高于 SeaORM 2 / SQLx 0.9 要求的 1.94。

## Goals / Non-Goals

**Goals:**

- 将 SeaORM、SeaQuery 和 SQLx 作为一次原子升级迁移到兼容版本。
- 保持现有 entity、relation、迁移文件、数据库结构和业务行为不变。
- 消除 workspace 中 SQLx 0.8 与 0.9 并存的风险。
- 让本地、CI、Docker 和 `sqlx migrate` 使用同一套 SQLx 0.9 工具链。
- 以现有测试和 SQL 生成断言证明升级没有改变查询语义。

**Non-Goals:**

- 不重写 entity 为 SeaORM 2 的新 `#[sea_orm::model]` 格式。
- 不采用 strongly-typed columns、nested ActiveModel、entity-first schema 等新功能。
- 不修改数据库 schema、迁移历史或业务规则。
- 不拆分或重构 repository 层；只做升级所需的机械适配。

## Decisions

### 1. 原子升级 SeaORM 与 SQLx

`ferriskey-core` 和 `ferriskey-api` 将同时升级到：

```text
sea-orm  2.0.4
sea-query 1.0.x（由 SeaORM 传递）
sqlx     0.9.0
```

原因是 SeaORM 的 `DatabaseConnection`、`PgPool`、`PgListener` 和 SQLx bind traits 必须来自同一个 `sqlx_core` 实例。单独升级任一侧都会产生大量 "multiple different versions of crate sqlx_core" 错误。

替代方案是暂时保留 SeaORM 1.x，或隔离两套数据库访问层；前者不能满足升级目标，后者会长期增加类型转换和维护成本。

### 2. 保持传统 entity 与 relation 格式

SeaORM 2 官方保留旧 entity 形式的兼容性。此次只验证现有 entity 在 2.0.4 下可编译、可查询，不在同一变更中重写 73 个 entity。

替代方案是同步迁移到新 entity 格式；它会显著扩大 diff、引入 relation 行为变化，并掩盖依赖升级的真实回归风险。

### 3. 按 API 类别做编译驱动迁移

预计修改集中在以下类别：

- 为 SeaQuery `Expr` 操作补齐 `use sea_orm::ExprTrait;`。
- 将 SeaORM 的 `Statement` executor 调用改为 `execute_raw`、`query_one_raw`、`query_all_raw` 或 `stream_raw`。
- 移除不再需要的 `IntoCondition` 转换，改用 `Condition` / `Expr` 的 `From` / `Into`。
- 对齐 SQLx 0.9 的 `PgPool`、`PgListener`、`ConnectOptions` 和 raw query 类型。
- 只在编译器或测试暴露问题时调整 iterator、condition 和 connection 类型。

已识别的高风险模块为：

- `core/src/infrastructure/authz/{principal_slice,repository}.rs`
- `core/src/infrastructure/compass/`
- `core/src/infrastructure/user/repository.rs`
- `core/src/infrastructure/webhook/repositories/webhook_delivery_repository.rs`
- `core/src/infrastructure/repositories/keystore_repository.rs`
- `core/src/application/mod.rs`
- `core/src/infrastructure/token_state/listener.rs`

不使用大范围搜索替换。每条替换都以编译错误和现有测试为依据，避免误改普通字符串或 `ColumnTrait` API。

### 4. SQLx 0.9 同步覆盖运行时、测试与工具链

直接依赖升级为 SQLx 0.9.0，保留 `runtime-tokio-rustls`、`postgres`、`migrate` feature。SeaORM 连接池重新成为唯一的 `PgPool` 来源。

Dockerfile 的 `sqlx-cli` 升级到 0.9.0，并删除 `--locked`，因为 SQLx 0.9 的 crate 不再提供可供 `cargo install --locked` 使用的 Cargo.lock。

替代方案是继续使用 SQLx 0.8 CLI；迁移文件本身通常兼容，但会让运行时库、类型系统和 CLI 版本长期漂移，不作为目标状态。

### 5. 先完成兼容升级，再评估 2.0 新能力

升级完成的定义是旧代码在 SeaORM 2 / SQLx 0.9 下行为不变，而不是采用 2.0 的所有新特性。新 entity 格式、strongly-typed columns 等能力留到后续独立变更，以便单独衡量收益和风险。

## Risks / Trade-offs

- [依赖图中仍残留 SQLx 0.8] -> 升级后检查 `Cargo.lock` 与 `cargo tree -d`，只有在确认唯一的 `sqlx-core` 版本后才完成变更。
- [SQLx 0.9 raw query trait 或类型推断变化] -> 用编译器和现有 PostgreSQL 测试逐项修复，不引入运行时类型转换。
- [SeaQuery 1.0 条件语义变化] -> 依赖已有 SQL 生成断言和数据库集成测试验证筛选、分页、排序和 `ILIKE` 行为。
- [连接池与监听器跨 crate 类型不一致] -> 优先迁移 `token_state/listener.rs`、Compass 批处理和 `application/mod.rs` 中传递 `PgPool` 的路径。
- [PostgreSQL identity/serial 默认值变化] -> 当前 schema 由手写迁移管理；保持迁移文件不变，并通过全新数据库迁移和测试确认行为。
- [SQLx CLI 安装参数失效] -> Dockerfile 固定 0.9.0 并移除 `--locked`，在镜像构建阶段验证 `sqlx --version` 和迁移命令。
- [大规模机械修改引入非预期行为] -> 保持 entity 与 schema 不变，分批按模块修复，最后运行完整 workspace 和 ignored PostgreSQL 测试。

## Migration Plan

1. 记录当前依赖图、编译和测试基线，并确认当前没有 SQLx 0.8 / 0.9 混合。
2. 在一个变更中更新 `core`、`api` 的 SeaORM / SQLx 版本、`Cargo.lock` 和 Dockerfile 的 SQLx CLI。
3. 按编译错误处理 SeaQuery 1.0、SeaORM raw executor 和 SQLx 0.9 类型适配。
4. 重点验证连接池、PostgreSQL listener、Compass 批处理、raw SQL 和 entity 查询。
5. 使用全新 PostgreSQL 数据库运行迁移与完整测试套件，构建 Docker 镜像并执行 `sqlx migrate`。
6. 全部通过后移除 Dependabot 对 SeaORM 2 / SQLx 0.9 的忽略。

本次变更不修改数据库 schema，因此部署回滚只需回到升级前的应用镜像和 Cargo 依赖；没有数据库迁移回滚步骤。
