use sea_orm::{
  ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, DatabaseTransaction, DbBackend, DbErr,
  DeleteResult, EntityTrait, ExecResult, IntoActiveModel, PrimaryKeyTrait, QueryResult, Statement,
};

#[derive(Clone)]
pub struct BaseRepository<C: ConnectionTrait> {
  conn: C,
}

impl<C: ConnectionTrait> BaseRepository<C> {
  pub fn new(conn: C) -> Self {
    Self { conn }
  }

  pub fn connection(&self) -> &C {
    &self.conn
  }

  pub async fn find_by_id<E, I>(&self, id: I) -> Result<Option<E::Model>, DbErr>
  where
    E: EntityTrait,
    I: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
  {
    E::find_by_id(id).one(self.connection()).await
  }

  pub async fn insert<A>(&self, active: A) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
  where
    A: ActiveModelTrait + ActiveModelBehavior + Send,
    A::Entity: EntityTrait,
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
  {
    active.insert(self.connection()).await
  }

  pub async fn update<A>(&self, active: A) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
  where
    A: ActiveModelTrait + ActiveModelBehavior + Send,
    A::Entity: EntityTrait,
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
  {
    active.update(self.connection()).await
  }

  pub async fn delete<A>(&self, active: A) -> Result<DeleteResult, DbErr>
  where
    A: ActiveModelTrait + ActiveModelBehavior + Send,
    A::Entity: EntityTrait,
  {
    active.delete(self.connection()).await
  }
}

#[derive(Clone, Copy)]
pub struct TransactionConnection<'a> {
  txn: &'a DatabaseTransaction,
}

impl<'a> TransactionConnection<'a> {
  pub fn new(txn: &'a DatabaseTransaction) -> Self {
    Self { txn }
  }
}

#[async_trait::async_trait]
impl ConnectionTrait for TransactionConnection<'_> {
  fn get_database_backend(&self) -> DbBackend {
    self.txn.get_database_backend()
  }

  async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
    self.txn.execute_raw(stmt).await
  }

  async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
    self.txn.execute_unprepared(sql).await
  }

  async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
    self.txn.query_one_raw(stmt).await
  }

  async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
    self.txn.query_all_raw(stmt).await
  }

  fn support_returning(&self) -> bool {
    self.txn.support_returning()
  }

  fn is_mock_connection(&self) -> bool {
    self.txn.is_mock_connection()
  }
}
