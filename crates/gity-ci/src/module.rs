use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use sea_orm::DatabaseConnection;

use crate::CiService;

fn service(connection: Shared<DatabaseConnection>) -> Result<Shared<CiService>, fluxdi::Error> {
  Ok(Shared::new(CiService::new((*connection).clone())))
}

// CI application bindings. The application supplies its shared SeaORM connection.
static_module! {
    pub struct CiModule {
        #[scope(root)]
        service: CiService = service(connection);
        external connection: DatabaseConnection;
    }
}

impl Module for CiModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = self.service()?;
    injector.provide(Provider::<CiService>::root(move |_| service.clone()));
    Ok(())
  }
}
