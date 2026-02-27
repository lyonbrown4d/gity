use crate::configuration::cfg::{CacheType, Config};
use fred::clients::Client;
use fred::prelude::{Builder, ClientLike, EventInterface, TcpConfig};
use migration::{Migrator, MigratorTrait};
use sadi::Injector;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AppState {
  pub injector: Arc<Injector>,
}

impl AppState {
  pub fn new(injector: Injector) -> AppState {
    AppState {
      injector: Arc::new(injector),
    }
  }
}

// pub fn create_app_state(injector: sadi::runtime::Shared<Injector>) -> AppState {
//   // // 1. 数据库连接
//   // let db_conn: DatabaseConnection = sea_orm::Database::connect(&cfg.database.url)
//   //     .await
//   //     .expect("Failed to connect to database");
//   //
//   // // 2. Redis 客户端初始化（如果是 MEMORY 或 None 就是 None）
//   // let redis_client = match cfg.cache.as_ref().map(|c| &c.cache_type) {
//   //   Some(CacheType::REDIS) => {
//   //     let cache_url = cfg.cache.as_ref().unwrap().url.clone();
//   //     let redis_config = fred::prelude::Config::from_url(&cache_url)
//   //         .expect("Failed to parse Redis URL");
//   //
//   //     let client = Builder::from_config(redis_config)
//   //         .with_connection_config(|c| {
//   //           c.connection_timeout = Duration::from_secs(5);
//   //           c.tcp.nodelay = Some(true);
//   //         })
//   //         .build()
//   //         .expect("Failed to build Redis client");
//   //
//   //     client.init().await.expect("Failed to init Redis client");
//   //
//   //     client.on_error(|(err, server)| async move {
//   //       eprintln!("Redis connection error: {:?} {:?}", server, err);
//   //       Ok(())
//   //     });
//   //
//   //     Some(client)
//   //   }
//   //   _ => None,
//   // };
//   //
//   // Migrator::up(&db_conn, None)
//   //     .await
//   //     .expect("Failed to run migrations up");
//
//   // 4. 返回 AppState
//   // AppState {
//   //   injector: Arc::new(injector),
//   // }
// }
