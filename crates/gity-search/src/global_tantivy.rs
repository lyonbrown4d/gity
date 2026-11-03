use crate::global::{CandidatePage, EngineQuery, ResourceKind, SearchDocument};
use anyhow::{Context, Result};
use fs2::FileExt;
use std::{fs::OpenOptions, path::Path, sync::Arc};
use tantivy::{
  Index, TantivyDocument, Term, doc,
  query::{BooleanQuery, Occur, Query, TermQuery},
  schema::{Field, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions},
  tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer},
};
use tokio::sync::{Mutex, OwnedMutexGuard, Semaphore};

mod query;

const MAX_CANDIDATES: usize = 10_000;
const TOKENIZER: &str = "gity_global_grams_v1";

#[derive(Clone)]
pub(super) struct TantivyEngine {
  index: Index,
  fields: Arc<Fields>,
  writer_lock: Arc<Mutex<()>>,
  query_slots: Arc<Semaphore>,
  _process_lock: Arc<std::fs::File>,
}

struct Fields {
  project: Field,
  namespace: Field,
  resource_id: Field,
  confidential: Field,
  kind: Field,
  key: Field,
  source: Field,
  title: Field,
  body: Field,
  path: Field,
  grams: Field,
}

impl TantivyEngine {
  pub fn new(root: &Path) -> Result<Self> {
    std::fs::create_dir_all(root)?;
    let root = root.canonicalize()?;
    let lock_path = root.join("global.lock");
    let lock = OpenOptions::new()
      .create(true)
      .truncate(false)
      .read(true)
      .write(true)
      .open(lock_path)?;
    FileExt::try_lock_exclusive(&lock).context("global Tantivy index is already in use")?;
    let path = root.join("global-index");
    let index = if path.join("meta.json").exists() {
      Index::open_in_dir(path)?
    } else {
      std::fs::create_dir_all(&path)?;
      let (schema, _) = schema();
      Index::create_in_dir(path, schema)?
    };
    let fields = schema_fields(&index)?;
    index.tokenizers().register(
      TOKENIZER,
      TextAnalyzer::builder(NgramTokenizer::new(1, 3, false)?)
        .filter(LowerCaser)
        .build(),
    );
    Ok(Self {
      index,
      fields: Arc::new(fields),
      writer_lock: Arc::new(Mutex::new(())),
      query_slots: Arc::new(Semaphore::new(8)),
      _process_lock: Arc::new(lock),
    })
  }

  pub async fn replace_project(
    &self,
    project_id: i64,
    namespace: &str,
    documents: Vec<SearchDocument>,
  ) -> Result<()> {
    let guard = self.writer_lock.clone().lock_owned().await;
    let index = self.index.clone();
    let fields = self.fields.clone();
    let namespace = namespace.to_owned();
    tokio::task::spawn_blocking(move || {
      replace_sync(&index, &fields, project_id, &namespace, documents, guard)
    })
    .await?
  }

  pub async fn delete_namespace(&self, project_id: i64, namespace: &str) -> Result<()> {
    let guard = self.writer_lock.clone().lock_owned().await;
    let index = self.index.clone();
    let fields = self.fields.clone();
    let namespace = namespace.to_owned();
    tokio::task::spawn_blocking(move || delete_sync(&index, &fields, project_id, &namespace, guard))
      .await?
  }

  pub async fn query(&self, request: EngineQuery) -> Result<CandidatePage> {
    let permit = self
      .query_slots
      .clone()
      .try_acquire_owned()
      .context("global Tantivy query capacity is busy")?;
    let index = self.index.clone();
    let fields = self.fields.clone();
    tokio::task::spawn_blocking(move || {
      let _permit = permit;
      query::search(&index, &fields, request)
    })
    .await?
  }
}

fn schema() -> (Schema, ()) {
  let mut builder = Schema::builder();
  builder.add_i64_field("project_id", STORED | tantivy::schema::INDEXED);
  builder.add_text_field("namespace", STRING | STORED);
  builder.add_text_field("kind", STRING | STORED);
  builder.add_text_field("key", STRING | STORED);
  builder.add_text_field("source", STORED);
  builder.add_text_field("title", STORED);
  builder.add_text_field("body", STORED);
  builder.add_text_field("path", STORED);
  builder.add_text_field("resource_id", STRING | STORED);
  builder.add_bool_field("confidential", STORED | tantivy::schema::INDEXED);
  builder.add_text_field(
    "grams",
    TextOptions::default().set_indexing_options(
      TextFieldIndexing::default()
        .set_tokenizer(TOKENIZER)
        .set_index_option(IndexRecordOption::Basic),
    ),
  );
  let schema = builder.build();
  (schema, ())
}

fn schema_fields(index: &Index) -> Result<Fields> {
  fields_from_schema(&index.schema())
}
fn fields_from_schema(schema: &Schema) -> Result<Fields> {
  Ok(Fields {
    project: schema.get_field("project_id")?,
    namespace: schema.get_field("namespace")?,
    resource_id: schema.get_field("resource_id")?,
    confidential: schema.get_field("confidential")?,
    kind: schema.get_field("kind")?,
    key: schema.get_field("key")?,
    source: schema.get_field("source")?,
    title: schema.get_field("title")?,
    body: schema.get_field("body")?,
    path: schema.get_field("path")?,
    grams: schema.get_field("grams")?,
  })
}

fn kind_name(kind: ResourceKind) -> &'static str {
  match kind {
    ResourceKind::Code => "code",
    ResourceKind::Issue => "issue",
  }
}
fn normalized_text(document: &SearchDocument) -> String {
  format!("{} {} {}", document.title, document.path, document.body).to_lowercase()
}

fn namespace_query(
  project: Field,
  namespace: Field,
  project_id: i64,
  token: &str,
) -> Box<dyn Query> {
  Box::new(BooleanQuery::new(vec![
    (
      Occur::Must,
      Box::new(TermQuery::new(
        Term::from_field_i64(project, project_id),
        IndexRecordOption::Basic,
      )),
    ),
    (
      Occur::Must,
      Box::new(TermQuery::new(
        Term::from_field_text(namespace, token),
        IndexRecordOption::Basic,
      )),
    ),
  ]))
}

fn replace_sync(
  index: &Index,
  fields: &Fields,
  project_id: i64,
  namespace: &str,
  documents: Vec<SearchDocument>,
  guard: OwnedMutexGuard<()>,
) -> Result<()> {
  let _guard = guard;
  let mut writer = index.writer_with_num_threads::<TantivyDocument>(1, 50_000_000)?;
  writer.delete_query(namespace_query(
    fields.project,
    fields.namespace,
    project_id,
    namespace,
  ))?;
  for document in documents {
    writer.add_document(doc!(
      fields.project => document.project_id,
      fields.namespace => namespace,
      fields.kind => kind_name(document.kind),
      fields.key => document.key,
      fields.resource_id => document.resource_id,
      fields.confidential => document.confidential,
      fields.source => serde_json::to_string(&document)?,
      fields.title => document.title,
      fields.body => document.body,
      fields.path => document.path,
      fields.grams => normalized_text(&document),
    ))?;
  }
  writer.commit()?;
  Ok(())
}

fn delete_sync(
  index: &Index,
  fields: &Fields,
  project_id: i64,
  namespace: &str,
  guard: OwnedMutexGuard<()>,
) -> Result<()> {
  let _guard = guard;
  let mut writer = index.writer_with_num_threads::<TantivyDocument>(1, 50_000_000)?;
  writer.delete_query(namespace_query(
    fields.project,
    fields.namespace,
    project_id,
    namespace,
  ))?;
  writer.commit()?;
  Ok(())
}
