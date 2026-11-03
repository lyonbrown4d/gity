use super::{Fields, MAX_CANDIDATES, kind_name, namespace_query};
use crate::global::{CandidatePage, EngineQuery, ResourceKind, SearchDocument, matches};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use tantivy::{
  Index, TantivyDocument, Term,
  collector::TopDocs,
  query::{BooleanQuery, Occur, Query, TermQuery},
  schema::{IndexRecordOption, Value},
};

pub(super) fn search(
  index: &Index,
  fields: &Fields,
  request: EngineQuery,
) -> Result<CandidatePage> {
  let query = request.query.trim();
  let chars = query.to_lowercase().chars().collect::<Vec<_>>();
  if chars.is_empty() {
    return Ok(CandidatePage::default());
  }
  let grams = if chars.len() < 3 {
    vec![chars.iter().collect::<String>()]
  } else {
    chars
      .windows(3)
      .map(|window| window.iter().collect::<String>())
      .collect::<BTreeSet<_>>()
      .into_iter()
      .collect()
  };
  let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
  let namespaces = request
    .namespaces
    .iter()
    .map(|(project, token)| {
      (
        Occur::Should,
        namespace_query(fields.project, fields.namespace, *project, token),
      )
    })
    .collect();
  clauses.push((Occur::Must, Box::new(BooleanQuery::new(namespaces))));
  let projects = request
    .project_ids
    .iter()
    .map(|id| {
      (
        Occur::Should,
        Box::new(TermQuery::new(
          Term::from_field_i64(fields.project, *id),
          IndexRecordOption::Basic,
        )) as Box<dyn Query>,
      )
    })
    .collect();
  clauses.push((Occur::Must, Box::new(BooleanQuery::new(projects))));
  clauses.push((
    Occur::Must,
    Box::new(TermQuery::new(
      Term::from_field_text(fields.kind, kind_name(request.kind)),
      IndexRecordOption::Basic,
    )),
  ));
  if request.kind == ResourceKind::Issue {
    let mut visibility: Vec<(Occur, Box<dyn Query>)> = vec![(
      Occur::Should,
      Box::new(TermQuery::new(
        Term::from_field_bool(fields.confidential, false),
        IndexRecordOption::Basic,
      )),
    )];
    visibility.extend(request.confidential_projects.iter().map(|id| {
      (
        Occur::Should,
        Box::new(TermQuery::new(
          Term::from_field_i64(fields.project, *id),
          IndexRecordOption::Basic,
        )) as Box<dyn Query>,
      )
    }));
    visibility.extend(request.confidential_issue_ids.iter().map(|id| {
      (
        Occur::Should,
        Box::new(TermQuery::new(
          Term::from_field_text(fields.resource_id, &id.to_string()),
          IndexRecordOption::Basic,
        )) as Box<dyn Query>,
      )
    }));
    clauses.push((Occur::Must, Box::new(BooleanQuery::new(visibility))));
  }
  for gram in grams {
    clauses.push((
      Occur::Must,
      Box::new(TermQuery::new(
        Term::from_field_text(fields.grams, &gram),
        IndexRecordOption::Basic,
      )),
    ));
  }
  let searcher = index.reader()?.searcher();
  let docs = searcher.search(
    &BooleanQuery::new(clauses),
    &TopDocs::with_limit(MAX_CANDIDATES).order_by_score(),
  )?;
  let overflow = docs.len() == MAX_CANDIDATES;
  let mut found = Vec::new();
  for (score, address) in docs {
    let stored: TantivyDocument = searcher.doc(address)?;
    let raw = stored
      .get_first(fields.source)
      .and_then(|value| value.as_str())
      .context("global index source missing")?;
    let mut document: SearchDocument = serde_json::from_str(raw)?;
    if matches(query, &document.title)
      || matches(query, &document.body)
      || matches(query, &document.path)
    {
      let title_boost = if matches(query, &document.title) {
        3.0
      } else {
        0.0
      };
      // Candidate pages retain identity and navigation metadata; the platform hydrates display text.
      document.title = String::new();
      document.body = String::new();
      found.push((score + title_boost, document));
    }
  }
  found.sort_by(|left, right| {
    right
      .0
      .total_cmp(&left.0)
      .then_with(|| left.1.key.cmp(&right.1.key))
  });
  let end = request.offset.saturating_add(request.limit);
  let has_more = found.len() > end;
  let partial = overflow && !has_more;
  let documents = found
    .into_iter()
    .skip(request.offset)
    .take(request.limit)
    .map(|(_, document)| document)
    .collect();
  Ok(CandidatePage {
    documents,
    has_more,
    partial,
  })
}
