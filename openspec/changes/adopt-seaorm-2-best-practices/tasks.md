# Tasks

Each numbered group is intended to become one pull request. A group may contain several implementation tasks, but it must remain limited to its SeaORM usage domain and must not include database migrations or business behavior changes.

## 1. Migrate entities to the dense SeaORM 2 format

- [x] 1.1 Record the SeaORM 2.0.4 / SeaQuery 1.0.x / SQLx 0.9.0 baseline and the 73-entity inventory; verify `cargo tree -d` contains no duplicate `sqlx-core` or `sea-orm` versions.
- [x] 1.2 Start a fresh PostgreSQL database, run all SQLx migrations, and generate dense entities with SeaORM CLI 2.0.4 into a temporary directory; verify the generated output uses `#[sea_orm::model]` and the complete migrated schema is discoverable.
- [x] 1.3 Compare every existing `Relation` variant and `Related` implementation with the generated dense entities; verify no relation, junction relation, or self-reference is dropped.
- [x] 1.4 Replace `core/src/entity/` with the reconciled dense entities and update `mod.rs` / `prelude.rs`; verify `cargo check --workspace --exclude ferriskey-client --locked` passes.
- [x] 1.5 Fix compile mismatches exposed by the schema-accurate entities, including the now non-null `clients.require_pkce` field; verify `cargo check -p ferriskey-core` and the affected client mapper/repository tests pass.
- [x] 1.6 Record `security_event_chain_heads` and `user_token_revocations` as deliberate codegen exceptions with the concurrency/performance reason; verify a second codegen run with the documented exception list produces no entity diff.
- [x] 1.7 Verify the Entity pull request does not introduce `Entity::COLUMN`, Entity Loader, Nested ActiveModel, transaction changes, or schema migrations by reviewing the diff and search results.
- [x] 1.8 Run workspace Clippy and the PostgreSQL-backed core/API integration suites; verify no Entity-format regression and no behavior change.

## 2. Adopt type-aware columns

- [x] 2.1 Inventory the approximately 688 `Entity::Column` references and classify each as a type-relevant operation or a homogeneous generic API; verify the classification covers every hit in `core/src` and test code.
- [x] 2.2 Replace type-relevant filters, comparisons, null checks, `IN` predicates, string helpers, and expression builders with `Entity::COLUMN`; verify `cargo check --workspace --exclude ferriskey-client --locked` passes.
- [x] 2.3 Keep `SortColumn<E>` and other heterogeneous abstractions on the `Column` enum or an equivalent type-erased expression; verify the pagination tests still generate the expected `ORDER BY` clauses.
- [x] 2.4 Add typed-column type assertions and a type-specific helper query test; verify the expected `StringColumn`, `BoolColumn`, and `DateTimeLikeColumn` types compile and the generated SQL is unchanged.
- [x] 2.5 Run SQL-generation assertions and PostgreSQL integration tests for changed queries; verify SQL semantics, ordering, and pagination are unchanged.
- [x] 2.6 Run workspace Clippy and the relevant unit/integration suites; verify the Column pull request contains no Entity format, loader, nested-write, or transaction changes.

## 3. Use transaction closures

- [x] 3.1 Inventory the 13 production SeaORM `begin` / `commit` sites plus one SQLx test-only site; verify every site is converted or explicitly exempted.
- [x] 3.2 Convert eligible keystore, authz, client, portal, realm, consent, and resource-owner transactions to `ConnectionTrait::transaction`; verify `cargo check --workspace --exclude ferriskey-client --locked` passes.
- [x] 3.3 Preserve the SQLx webhook-delivery test transaction because it explicitly holds a row lock to test worker concurrency; verify it is the only remaining `begin` / `commit` site.
- [x] 3.4 Reuse the existing rollback, stale-version, secret-rotation, consent, authz, and portal lifecycle tests for converted transactions; verify each converted path has rollback or failure coverage.
- [x] 3.5 Run existing row-lock and concurrency tests for keystore and webhook delivery; verify isolation, lock scope, and error mapping remain unchanged.
- [x] 3.6 Run workspace Clippy and PostgreSQL integration tests; verify the transaction pull request does not include raw SQL rewrites, Entity changes, or nested ActiveModel adoption.

## 4. Use Entity Loader for related reads

- [ ] 4.1 Inventory the five current `find_with_related` / `find_also_related` sites and evaluate each for Entity Loader suitability; verify each site is marked for conversion or explicitly retained with a reason.
- [ ] 4.2 Convert the selected role, user, client, and Compass flow object-graph reads to Entity Loader; verify `cargo check -p ferriskey-core --locked` passes.
- [ ] 4.3 Preserve deterministic ordering for one-to-many and many-to-many loaded relations; verify loader tests assert the expected relation order.
- [ ] 4.4 Add query-count or equivalent observable assertions for converted reads; verify converted object-graph loads avoid N+1 queries.
- [ ] 4.5 Verify raw batch and aggregate paths, including Compass `UNNEST` writes and SeaWatch chain writes, are untouched.
- [ ] 4.6 Run the affected repository, API, and PostgreSQL integration tests plus workspace Clippy; verify returned object graphs and empty-relation semantics are unchanged.

## 5. Use Nested ActiveModel for selected aggregate writes

- [ ] 5.1 Select aggregate-write candidates using the design criteria: one aggregate root, parent/child persistence order, foreign-key backfill, and existing behavioral tests; verify optimized raw paths and token hot paths are excluded.
- [ ] 5.2 Convert the selected parent/child writes to Nested ActiveModel; verify `cargo check --workspace --exclude ferriskey-client --locked` passes.
- [ ] 5.3 Preserve append/replace behavior, conflict handling, cascade rules, and statement-level locking where applicable; verify repository tests cover each retained semantic.
- [ ] 5.4 Add idempotence and atomicity tests for each converted aggregate save; verify a repeated save is a no-op unless replace/delete semantics explicitly request a change.
- [ ] 5.5 Verify Compass batch writes, webhook delivery outbox, SeaWatch chain writes, and token/access-token paths remain on their existing SQL implementations.
- [ ] 5.6 Run workspace Clippy and PostgreSQL integration tests; verify no public API or domain behavior changed.

## 6. Standardize raw SQL and connection boundaries

- [ ] 6.1 Inventory raw `Statement` sites and repository connection fields; classify each as PostgreSQL-specific, performance-sensitive, reusable helper, or safely convertible; verify every site is accounted for.
- [ ] 6.2 Preserve the SQL-only status and rationale for `security_event_chain_heads` and `user_token_revocations`; verify no ORM CRUD path is introduced for either table.
- [ ] 6.3 Adopt `DbErr::sql_err` / `SqlErr` for common constraint-error classification where it replaces fragile string matching; verify the affected repository tests pass.
- [ ] 6.4 Change reusable helpers that execute on behalf of callers from `DatabaseConnection` to `ConnectionTrait` where they must accept both a pool and a transaction; verify both call forms compile.
- [ ] 6.5 Adopt `raw_sql!` only where it improves parameter safety or readability without changing SQL; verify generated SQL and query budgets remain unchanged.
- [ ] 6.6 Run workspace Clippy and PostgreSQL integration tests; verify no CTE, `UNNEST`, locking, audit-chain, or token-hot-path SQL was rewritten for style.

## 7. Integration verification

- [ ] 7.1 Run the full workspace compile, Clippy, and non-PostgreSQL test suite after all six pull requests; verify all pass with warnings denied.
- [ ] 7.2 Run the complete PostgreSQL-backed integration suite on a database built only from SQLx migrations; verify migrations, Entity codegen, query behavior, query budgets, and concurrency tests pass.
- [ ] 7.3 Verify no database schema, public API, domain behavior, migration history, or unsupported SeaORM 2 feature was introduced by the six pull requests.

## Workflow follow-up

- Archive this change after all six pull requests are merged and the integration verification passes.
- Record the completed SeaORM 2 best-practice work on the relevant tracking issue or pull request.
