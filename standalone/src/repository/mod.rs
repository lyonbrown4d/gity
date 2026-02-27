use crate::configuration::cfg::Config;
use sadi::{Injector, Module, Provider, Shared};
use sea_orm::DatabaseConnection;
use tokio::runtime::Handle;

pub struct DatabaseRepositoryModule;

impl Module for DatabaseRepositoryModule {
  fn providers(&self, _injector: &Injector) {
    let cfg = _injector.resolve::<Config>();
    // 1. 数据库连接
    let db_conn: DatabaseConnection = Handle::current()
      .block_on(async { sea_orm::Database::connect(&cfg.database.url).await })
      .unwrap();

    // _injector.provide::<DatabaseConnection>(Provider::root(|_injector| {
    //   <Shared<DatabaseConnection>>::from(db_conn)
    // }));
  }
}
