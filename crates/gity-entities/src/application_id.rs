//! Application-generated primary keys for SeaORM entities.
//!
//! SeaORM has no pluggable identifier generator: the only persist-time hook is
//! `ActiveModelBehavior::before_save`. Every entity therefore needs the same
//! impl, so entities declare it once through [`application_ids!`] instead of
//! repeating the body in 39 files.

/// Assigns a Snowflake identifier to each entity row that does not carry one.
///
/// Identifiers generated ahead of an insert (parent ids, storage keys) are
/// already set and are left untouched.
macro_rules! application_ids {
  ($($entity:ident),* $(,)?) => {
    $(
      #[::async_trait::async_trait]
      impl ::sea_orm::ActiveModelBehavior for $entity::ActiveModel {
        async fn before_save<C>(
          mut self,
          _db: &C,
          insert: bool,
        ) -> ::std::result::Result<Self, ::sea_orm::DbErr>
        where
          C: ::sea_orm::ConnectionTrait,
        {
          if insert && self.id.is_not_set() {
            self.id = ::sea_orm::ActiveValue::Set(
              ::gity_ids::try_new_id().map_err(|error| ::sea_orm::DbErr::Custom(error.to_string()))?,
            );
          }
          Ok(self)
        }
      }
    )*
  };
}

pub(crate) use application_ids;
