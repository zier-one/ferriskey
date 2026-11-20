# Tasks

## 1. 建立升级基线

- [x] 1.1 记录当前 `Cargo.lock` 中的 SeaORM、SeaQuery、SQLx 版本，并运行 `cargo tree -d`；确认变更前不存在 SQLx 0.9 混合依赖。
- [x] 1.2 使用当前代码运行一次 workspace 编译与测试二进制构建（至少 `cargo check --workspace --exclude ferriskey-client --locked` 与 `cargo test --workspace --exclude ferriskey-client --lib --bins --tests --locked --no-run`）；记录升级前基线结果。

## 2. 更新依赖与工具链

- [x] 2.1 将 `core/Cargo.toml` 中的 `sea-orm` 更新到 `2.0.4`、`sqlx` 更新到 `0.9.0`，保留 PostgreSQL、runtime、migrate、macros 和 UUID features；验证 manifest 中不再声明 SeaORM 1.x 或 SQLx 0.8.x。
- [x] 2.2 将 `api/Cargo.toml` 的正常依赖与 dev-dependency 更新到相同的 SeaORM 2.0.4 / SQLx 0.9.0 版本和必要 features；验证 manifest 中不再声明旧版本。
- [x] 2.3 将 Dockerfile 中的 `sqlx-cli` 更新到 `0.9.0` 并移除不再可用的 `--locked`，然后构建 `chef` 阶段验证 CLI 安装成功且 `sqlx --version` 输出 0.9.0。
- [x] 2.4 重新生成 `Cargo.lock` 并检查 `cargo tree -d` 与 lockfile；验证只存在一个 `sqlx-core` / `sqlx-postgres` 版本，且 SeaORM 为 2.0.4。

## 3. 适配 SeaORM 2 与 SeaQuery 1.0

- [x] 3.1 将 SeaORM 接收 `Statement` 的 `execute`、`query_one`、`query_all`、`stream` 调用迁移到对应的 `*_raw` API（至少覆盖 user、refresh token、access token、webhook、compass 和 secrets backfill）；验证搜索不再命中旧 executor 调用形式。
- [x] 3.2 为使用 SeaQuery `Expr` 方法的位置补齐 `ExprTrait`，并移除不再需要的 `IntoCondition` / `ConditionExpression` 用法；验证搜索不再命中已移除 API，相关查询构造测试通过。
- [x] 3.3 依据编译器错误修复 authz、Compass、user、webhook、keystore 与 `application/mod.rs` 中的 condition、iterator 和 connection 类型变化；验证 `cargo check -p ferriskey-core` 不再在这些模块报告错误。
- [x] 3.4 验证 `ferriskey-core` 全部 entity 在传统 `Relation` / `Related` 形式下可编译；验证 `cargo check -p ferriskey-core` 通过且 entity 未被改写为新格式。
- [x] 3.5 使用 SeaORM 2.0.4 CLI 从测试数据库生成一份临时 entity 参考输出，对比现有 entity 的表、列、类型和关系覆盖；验证不存在需要改变数据库契约的差异。

## 4. 适配 SQLx 0.9 运行时与测试

- [x] 4.1 更新 `core/src/infrastructure/db/postgres.rs` 与 `core/src/infrastructure/token_state/listener.rs` 的连接池和 listener 类型，确保 `get_postgres_connection_pool()` 直接供 SQLx 0.9 使用；验证 token-state 单元测试和 PostgreSQL listener 集成测试通过。
- [x] 4.2 更新 Compass 批量写入对 SeaORM 连接池和 SQLx 0.9 query binding 的使用；验证 Compass repository 测试与批量写入 SQL 断言通过。
- [x] 4.3 更新 `core` 与 `api` 中测试夹具、schema 创建、seed 和断言使用的 SQLx 0.9 调用；验证 `cargo test -p ferriskey-core --lib --tests --no-run` 与 `cargo test -p ferriskey-api --lib --bins --tests --no-run` 通过。
- [x] 4.4 检查 `sqlx-query` 宏、`PgListener`、`PgPoolOptions` 和 migration 调用在 SQLx 0.9 下的编译与运行行为；验证相关测试通过且没有引入 SQLx 0.8 类型转换层。

## 5. 集成验证与清理

- [x] 5.1 运行 `cargo clippy --workspace --exclude ferriskey-client --all-targets --locked -- -D warnings`，验证编译告警与 lint 全部清零。
- [x] 5.2 在全新 PostgreSQL 数据库上运行迁移并执行 `sh/rust-tests.sh`，验证全部 ignored/数据库测试通过，尤其覆盖 authz、Compass、webhook、keystore、分页和 token-state listener。
- [x] 5.3 构建 API Docker 镜像目标，验证镜像内 `sqlx --version` 为 0.9.0，并针对全新数据库执行 `sqlx migrate run` 成功。
- [x] 5.4 移除 `.github/dependabot.yml` 中对 SeaORM `>=2` 和 SQLx `>=0.9` 的忽略，验证配置可解析且没有其他旧版本约束。
- [x] 5.5 运行 `openspec validate "migrate-sea-orm-2-sqlx-0-9" --type change --strict`，并复查 `git diff` 仅包含依赖升级、必要适配、验证调整和 Dependabot 清理。

## Workflow follow-up

- 在项目审查要求满足后归档该变更。
- 更新或关闭上游跟踪项 #1758，并记录实际升级结果。
