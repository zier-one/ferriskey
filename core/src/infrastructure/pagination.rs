use sea_orm::ExprTrait;
use sea_orm::{
    Condition, ConnectionTrait, DbErr, EntityTrait, IntoSimpleExpr, Iterable, Order,
    PaginatorTrait, PrimaryKeyToColumn, QueryOrder, QuerySelect, Select,
    sea_query::{LikeExpr, SimpleExpr, extension::postgres::PgExpr},
};

use crate::domain::common::pagination::{DateRange, PageRequest, SortOrder};

pub trait SortColumn<E: EntityTrait> {
    fn column(&self) -> E::Column;
}

pub trait SortExpr {
    fn expr(&self) -> SimpleExpr;
}

pub fn direction(order: SortOrder) -> Order {
    match order {
        SortOrder::Asc => Order::Asc,
        SortOrder::Desc => Order::Desc,
    }
}

fn window<Q: QuerySelect, F, S>(query: Q, request: &PageRequest<F, S>) -> Q {
    query
        .offset(request.offset())
        .limit(u64::from(request.limit.get()))
}

pub fn page_by_expr<Q, F, S>(query: Q, request: &PageRequest<F, S>, tie_breaker: SimpleExpr) -> Q
where
    Q: QueryOrder + QuerySelect,
    S: SortExpr,
{
    let order = direction(request.sort.order);
    window(
        query
            .order_by(request.sort.field.expr(), order.clone())
            .order_by(tie_breaker, order),
        request,
    )
}

pub async fn paginate_also<E, R, F, S, C>(
    db: &C,
    select: Select<E>,
    related: R,
    tie_breaker: SimpleExpr,
    request: &PageRequest<F, S>,
) -> Result<(Vec<(E::Model, Option<R::Model>)>, u64), DbErr>
where
    E: EntityTrait,
    E::Model: Sync,
    R: EntityTrait,
    R::Model: Sync,
    S: SortExpr,
    C: ConnectionTrait,
{
    let total = select.clone().count(db).await?;
    let rows = page_by_expr(select.select_also(related), request, tie_breaker)
        .all(db)
        .await?;
    Ok((rows, total))
}

pub fn escape_like(value: &str) -> String {
    let mut pattern = String::with_capacity(value.len() + 2);
    pattern.push('%');
    for c in value.chars() {
        if matches!(c, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');
    pattern
}

pub fn contains<C: IntoSimpleExpr>(column: C, value: &str) -> SimpleExpr {
    column
        .into_simple_expr()
        .ilike(LikeExpr::new(escape_like(value)))
}

pub fn within<C: IntoSimpleExpr>(column: C, range: &DateRange) -> Condition {
    let column = column.into_simple_expr();
    Condition::all()
        .add_option(range.from.map(|from| column.clone().gte(from)))
        .add_option(range.to.map(|to| column.clone().lt(to)))
}

pub fn within_naive<C: IntoSimpleExpr>(column: C, range: &DateRange) -> Condition {
    let column = column.into_simple_expr();
    Condition::all()
        .add_option(range.from.map(|from| column.clone().gte(from.naive_utc())))
        .add_option(range.to.map(|to| column.clone().lt(to.naive_utc())))
}

pub fn order<E, F, S>(select: Select<E>, request: &PageRequest<F, S>) -> Select<E>
where
    E: EntityTrait,
    S: SortColumn<E>,
{
    let order = direction(request.sort.order);
    E::PrimaryKey::iter().fold(
        select.order_by(request.sort.field.column(), order.clone()),
        |select, key| select.order_by(key.into_column(), order.clone()),
    )
}

pub async fn paginate<E, F, S, C>(
    db: &C,
    select: Select<E>,
    request: &PageRequest<F, S>,
) -> Result<(Vec<E::Model>, u64), DbErr>
where
    E: EntityTrait,
    E::Model: Sync,
    S: SortColumn<E>,
    C: ConnectionTrait,
{
    let total = select.clone().count(db).await?;
    let models = window(order(select, request), request).all(db).await?;
    Ok((models, total))
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait, sea_query::Expr};

    use super::*;
    use chrono::{TimeZone, Utc};

    use crate::domain::common::pagination::{
        DateRange, PageLimit, PageNumber, PageRequest, Sort, SortOrder,
    };
    use crate::entity::{organizations, realms, users};

    #[derive(Debug, Clone, Copy, Default)]
    enum TestSort {
        Name,
        #[default]
        CreatedAt,
    }

    impl SortColumn<realms::Entity> for TestSort {
        fn column(&self) -> realms::Column {
            match self {
                TestSort::Name => realms::Column::Name,
                TestSort::CreatedAt => realms::Column::CreatedAt,
            }
        }
    }

    fn request(field: TestSort, order: SortOrder) -> PageRequest<(), TestSort> {
        PageRequest {
            page: PageNumber::default(),
            limit: PageLimit::default(),
            sort: Sort { field, order },
            filter: (),
        }
    }

    #[test]
    fn escape_like_wraps_and_escapes() {
        assert_eq!(escape_like("jo"), "%jo%");
        assert_eq!(escape_like("50%_a\\b"), "%50\\%\\_a\\\\b%");
    }

    #[test]
    fn order_appends_primary_key_in_the_same_direction() {
        let sql = order(
            realms::Entity::find(),
            &request(TestSort::Name, SortOrder::Asc),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            str::ends_with(&sql, r#"ORDER BY "realms"."name" ASC, "realms"."id" ASC"#),
            "{sql}"
        );
    }

    #[test]
    fn default_order_is_created_at_desc() {
        let sql = order(
            realms::Entity::find(),
            &request(TestSort::default(), SortOrder::default()),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            str::ends_with(
                &sql,
                r#"ORDER BY "realms"."created_at" DESC, "realms"."id" DESC"#
            ),
            "{sql}"
        );
    }

    impl SortExpr for TestSort {
        fn expr(&self) -> SimpleExpr {
            match self {
                TestSort::Name => Expr::col((users::Entity, users::COLUMN.username)),
                TestSort::CreatedAt => Expr::col((realms::Entity, realms::COLUMN.created_at)),
            }
        }
    }

    #[test]
    fn page_by_expr_orders_by_the_expression_then_the_tie_breaker_and_pages() {
        let request = PageRequest {
            page: PageNumber::try_from(3).expect("valid page"),
            limit: PageLimit::try_from(10).expect("valid limit"),
            sort: Sort {
                field: TestSort::Name,
                order: SortOrder::Asc,
            },
            filter: (),
        };
        let sql = page_by_expr(
            realms::Entity::find().select_also(users::Entity),
            &request,
            Expr::col((realms::Entity, realms::COLUMN.id)),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            str::contains(&sql, r#""users"."username" AS "B_username""#),
            "{sql}"
        );
        assert!(
            str::ends_with(
                &sql,
                r#"ORDER BY "users"."username" ASC, "realms"."id" ASC LIMIT 10 OFFSET 20"#
            ),
            "{sql}"
        );

        let sql = page_by_expr(
            realms::Entity::find(),
            &self::request(TestSort::CreatedAt, SortOrder::Desc),
            Expr::col((realms::Entity, realms::COLUMN.id)),
        )
        .build(DbBackend::Postgres)
        .to_string();
        assert!(
            str::ends_with(
                &sql,
                r#"ORDER BY "realms"."created_at" DESC, "realms"."id" DESC LIMIT 20 OFFSET 0"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn contains_is_a_qualified_case_insensitive_escaped_match() {
        let sql = realms::Entity::find()
            .filter(contains(realms::COLUMN.name, "a_b"))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(
            str::contains(&sql, r#""realms"."name" ILIKE E'%a\\_b%'"#),
            "{sql}"
        );
        assert!(!str::contains(&sql, "ESCAPE"), "{sql}");
    }

    #[test]
    fn within_naive_bounds_from_inclusive_to_exclusive() {
        let from = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single();
        let to = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single();
        let sql = realms::Entity::find()
            .filter(within_naive(
                realms::COLUMN.created_at,
                &DateRange::new(from, to),
            ))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(
            str::contains(
                &sql,
                r#""realms"."created_at" >= '2026-01-01 00:00:00.000000'"#
            ),
            "{sql}"
        );
        assert!(
            str::contains(
                &sql,
                r#""realms"."created_at" < '2026-02-01 00:00:00.000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn within_bounds_a_timestamptz_column() {
        let from = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single();
        let to = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single();
        let sql = organizations::Entity::find()
            .filter(within(
                organizations::COLUMN.created_at,
                &DateRange::new(from, to),
            ))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(
            str::contains(
                &sql,
                r#""organizations"."created_at" >= '2026-01-01 00:00:00.000000 +00:00'"#
            ),
            "{sql}"
        );
        assert!(
            str::contains(
                &sql,
                r#""organizations"."created_at" < '2026-02-01 00:00:00.000000 +00:00'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn unbounded_range_adds_no_condition() {
        let sql = realms::Entity::find()
            .filter(within_naive(
                realms::COLUMN.created_at,
                &DateRange::default(),
            ))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(str::ends_with(&sql, "WHERE TRUE"), "{sql}");
        assert!(!str::contains(&sql, r#""realms"."created_at" >"#), "{sql}");
        assert!(!str::contains(&sql, r#""realms"."created_at" <"#), "{sql}");
    }

    #[test]
    fn typed_columns_keep_their_sql_types_and_type_specific_helpers() {
        use sea_orm::{BoolColumn, DateTimeLikeColumn, StringColumn};

        let _: StringColumn<users::Entity> = users::COLUMN.username;
        let _: BoolColumn<users::Entity> = users::COLUMN.enabled;
        let _: DateTimeLikeColumn<users::Entity> = users::COLUMN.created_at;

        let sql = users::Entity::find()
            .filter(users::COLUMN.username.contains("Bob"))
            .build(DbBackend::Postgres)
            .to_string();
        assert!(
            str::contains(&sql, r#""users"."username" LIKE '%Bob%'"#),
            "{sql}"
        );
    }
}
