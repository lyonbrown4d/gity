mod logging;

use crate::bootstrap::logging::init_logging;
use crate::configuration::ConfigurationModule;
use sadi::{Application, Module};
use crate::repository::DatabaseRepositoryModule;

pub struct RootModule;

impl Module for RootModule {
  fn imports(&self) -> Vec<Box<dyn Module>> {
    vec![
      Box::new(ConfigurationModule),
      Box::new(DatabaseRepositoryModule),
    ]
  }

  fn providers(&self, injector: &sadi::Injector) {
    // // Register DatabaseService as singleton
    // injector.provide::<DatabaseService>(Provider::root(|_| {
    //     Shared::new(DatabaseService::new())
    // }));
    //
    // // Register UserService with DatabaseService dependency
    // injector.provide::<UserService>(Provider::root(|inj| {
    //     let db = inj.resolve::<DatabaseService>();
    //     UserService::new(db).into()
    // }));
  }
}
pub fn bootstrap() -> Result<Application, String> {
  init_logging();
  let mut app = Application::new(RootModule);

  app.bootstrap();

  Ok(app)
}
