use gity_entities::users;
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, sea_query::Expr};

/// Serialize credential creation with account deletion on every supported backend.
/// SQLite needs a write transaction; SELECT FOR UPDATE is unavailable there.
/// Preserve timestamps and do not depend on matched-vs-changed row-count semantics.
pub(super) async fn lock_user<C: ConnectionTrait>(
  connection: &C,
  id: i64,
) -> Result<Option<users::Model>, DbErr> {
  users::Entity::update_many()
    .col_expr(
      users::Column::UpdatedAt,
      Expr::col(users::Column::UpdatedAt),
    )
    .filter(users::Column::Id.eq(id))
    .exec(connection)
    .await?;
  users::Entity::find_by_id(id).one(connection).await
}
