use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use sea_orm::DatabaseConnection;

use crate::IdentityRepository;

fn repository(
  connection: Shared<DatabaseConnection>,
) -> Result<Shared<IdentityRepository>, fluxdi::Error> {
  Ok(Shared::new(IdentityRepository::new((*connection).clone())))
}

// Identity persistence bindings. The application supplies its shared SeaORM connection.
static_module! {
    pub struct IdentityModule {
        #[scope(root)]
        repository: IdentityRepository = repository(connection);
        external connection: DatabaseConnection;
    }
}

impl Module for IdentityModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let repository = self.repository()?;
    injector.provide(Provider::<IdentityRepository>::root(move |_| {
      repository.clone()
    }));
    Ok(())
  }
}
