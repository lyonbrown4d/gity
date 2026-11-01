use anyhow::Result;
use tantivy::{
  Index,
  schema::{Field, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions},
  tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer},
};
pub struct Fields {
  pub path: Field,
  pub content: Field,
  pub candidate: Field,
}
pub fn schema() -> (Schema, Fields) {
  let mut builder = Schema::builder();
  let path = builder.add_text_field("path", STRING | STORED);
  let content = builder.add_text_field("content", STORED);
  let candidate = builder.add_text_field(
    "candidate",
    TextOptions::default().set_stored().set_indexing_options(
      TextFieldIndexing::default()
        .set_tokenizer("gity_trigrams_v1")
        .set_index_option(IndexRecordOption::Basic),
    ),
  );
  (
    builder.build(),
    Fields {
      path,
      content,
      candidate,
    },
  )
}
pub fn register(index: &Index) -> Result<()> {
  let analyzer = TextAnalyzer::builder(NgramTokenizer::new(3, 3, false)?)
    .filter(LowerCaser)
    .build();
  index.tokenizers().register("gity_trigrams_v1", analyzer);
  Ok(())
}
pub fn fields(index: &Index) -> Result<Fields> {
  Ok(Fields {
    path: index.schema().get_field("path")?,
    content: index.schema().get_field("content")?,
    candidate: index.schema().get_field("candidate")?,
  })
}
