use super::*;

impl Db {
  /// Initialize or advance the database through the shared official migrator.
  pub async fn migrate(&self) -> Result<(), AppError> {
    gity_migrations::apply(&self.connection)
      .await
      .map_err(|error| match error {
        gity_migrations::MigrationError::Database(error) => error.into(),
        error => AppError::conflict(error.to_string()),
      })
  }
}

#[cfg(test)]
mod tests;
