use crate::CollaborationService;
use fluxdi::module::Module;
use fluxdi::{Injector, Provider, Shared, static_module};
use sea_orm::DatabaseConnection;
fn service(
  connection: Shared<DatabaseConnection>,
) -> Result<Shared<CollaborationService>, fluxdi::Error> {
  Ok(Shared::new(CollaborationService::new(
    (*connection).clone(),
  )))
}
static_module! {
  pub struct CollaborationModule {
    #[scope(root)]
    service: CollaborationService = service(connection);
    external connection: DatabaseConnection;
  }
}
impl Module for CollaborationModule {
  fn configure(&self, injector: &Injector) -> Result<(), fluxdi::Error> {
    let service = self.service()?;
    injector.provide(Provider::<CollaborationService>::root(move |_| {
      service.clone()
    }));
    Ok(())
  }
}
