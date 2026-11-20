# Proposal

## Why

FerrisKey now compiles on SeaORM 2.0.4 and SQLx 0.9.0, but most persistence code still follows SeaORM 1.x patterns: 72 expanded entities, one compact entity, no type-aware columns, manual relationship loading, and manual transaction handling. This proposal aligns the existing code with SeaORM 2's supported and recommended usage without changing database schemas, APIs, or business behavior.

The work must be delivered incrementally because each concern has a different risk profile. The project contribution rules require small, single-concern pull requests, so the migration is organized into one pull request per SeaORM usage domain.

## What Changes

- Convert the existing Entity layer to SeaORM 2's dense `#[sea_orm::model]` format, while keeping the two raw-SQL-only internal tables (`security_event_chain_heads` and `user_token_revocations`) as explicit codegen exceptions.
- Replace type-relevant `Entity::Column` usages with type-aware `Entity::COLUMN` columns, while preserving the `Column` enum where a homogeneous return type is required by generic pagination and sorting.
- Convert suitable manual `begin` / `commit` transactions to SeaORM's closure-based transaction API.
- Replace suitable `find_with_related` and `find_also_related` reads with Entity Loader queries.
- Use Nested ActiveModel only for writes that represent a true aggregate operation; keep high-throughput and concurrency-sensitive raw SQL paths unchanged.
- Standardize new raw SQL and connection abstractions where it improves safety or reuse, without rewriting PostgreSQL-specific `UNNEST`, CTE, locking, or audit-chain SQL.
- Keep SQLx migrations as the schema source of truth and do not adopt Entity First or schema sync.

### Pull Request Sequence

1. Migrate entities to the dense SeaORM 2 format.
2. Adopt type-aware columns.
3. Use transaction closures.
4. Use Entity Loader for related reads.
5. Use Nested ActiveModel for selected aggregate writes.
6. Standardize raw SQL and connection boundaries where beneficial.

## Capabilities

### New Capabilities

None. This is a persistence-layer refactor with no new behavior.

### Modified Capabilities

None. Existing API, domain behavior, database contracts, and schema remain unchanged. The change opts out of spec deltas with `skip_specs: true`.

## Impact

- `core/src/entity/`
- `core/src/infrastructure/`
- Query construction tests and PostgreSQL integration fixtures
- SeaORM version and entity-codegen validation paths
- No database schema changes
- No public API or business behavior changes
- No adoption of Entity First, RBAC, Arrow/Parquet, SQL Server, or `sea-orm-sync`
