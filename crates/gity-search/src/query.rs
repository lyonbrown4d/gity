use crate::*;
use gity_git::{SearchParams, SearchPlan};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use tantivy::{
    TantivyDocument, Term,
    collector::TopDocs,
    query::{BooleanQuery, Occur, Query, TermQuery},
    schema::{IndexRecordOption, Value as DocumentValue},
};
impl SearchIndex {
    /// None means the index is stale, incomplete, busy, or the query needs a live
    /// scan. Index errors are surfaced so the caller can log a safe fallback.
    pub async fn query(
        &self,
        project_id: i64,
        repo: &str,
        reference: &str,
        default_branch: &str,
        input: SearchParams,
    ) -> Result<Option<Value>> {
        let plan = SearchPlan::new(&input)?;
        if !self.path(project_id)?.join("meta.json").exists() {
            return Ok(None);
        }
        let query = input.query.trim();
        if input.regex
            || !query.is_ascii()
            || query.len() < 3
            || !["", default_branch].contains(&reference)
                && reference != format!("refs/heads/{default_branch}")
        {
            return Ok(None);
        }
        let revision = self.git.resolve_revision(repo, default_branch, "").await?;
        let service = self.clone();
        let query = query.to_ascii_lowercase();
        tokio::task::spawn_blocking(move || service.search(project_id, &revision, &query, plan))
            .await?
    }
    fn search(
        &self,
        project_id: i64,
        revision: &str,
        query: &str,
        plan: SearchPlan,
    ) -> Result<Option<Value>> {
        let Some(_lock) = self.lock(project_id, false)? else {
            return Ok(None);
        };
        let path = self.path(project_id)?;
        if !path.join("meta.json").exists() {
            return Ok(None);
        }
        let index = open(&path)?;
        let Some(metadata) = metadata(&index)? else {
            return Ok(None);
        };
        if !same_limits(&metadata, self.limits)
            || metadata.status.revision != revision
            || metadata.status.truncated
            || plan.max_file_size > metadata.max_file_size
        {
            return Ok(None);
        }
        let fields = schema::fields(&index)?;
        let terms = query
            .as_bytes()
            .windows(3)
            .map(|word| String::from_utf8_lossy(word).into_owned())
            .collect::<BTreeSet<_>>();
        let queries = terms
            .into_iter()
            .map(|term| {
                (
                    Occur::Must,
                    Box::new(TermQuery::new(
                        Term::from_field_text(fields.candidate, &term),
                        IndexRecordOption::Basic,
                    )) as Box<dyn Query>,
                )
            })
            .collect();
        let query = BooleanQuery::new(queries);
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let hits = searcher.search(
            &query,
            &TopDocs::with_limit(self.limits.max_files).order_by_score(),
        )?;
        let mut documents = Vec::new();
        for (_, address) in hits {
            let document: TantivyDocument = searcher.doc(address)?;
            let path = document
                .get_first(fields.path)
                .and_then(|v| v.as_str())
                .context("search index document lacks path")?
                .to_owned();
            if !plan.in_scope(&path) {
                continue;
            }
            documents.push((path, address));
        }
        documents.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        let mut results = Vec::new();
        let mut visited = 0;
        for (path, address) in documents {
            let document: TantivyDocument = searcher.doc(address)?;
            let content = document
                .get_first(fields.content)
                .and_then(|v| v.as_str())
                .context("search index document lacks content")?;
            if content.len() > plan.max_file_size {
                continue;
            }
            if visited >= plan.max_files {
                break;
            }
            visited += 1;
            plan.append_matches(&path, content.as_bytes(), &mut results);
            if results.len() >= plan.limit {
                break;
            }
        }
        Ok(Some(json!(results)))
    }
}
