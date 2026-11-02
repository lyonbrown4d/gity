use super::*;
pub(crate) async fn pipeline_detail(s: &AppState, p: i64, i: i64) -> Result<Value, AppError> {
  let row = pipeline_row(s, p, i).await?;
  let spec = config::parse(text(&row, "config_content"))?;
  Ok(json!({"pipeline":row,"spec":spec,"jobs":pipeline_jobs(s,p,i).await?}))
}
pub(crate) async fn pipeline_jobs(s: &AppState, p: i64, i: i64) -> Result<Value, AppError> {
  let rows = s
    .ci
    .pipeline_jobs(p, i)
    .await?
    .into_iter()
    .map(|(link, job)| {
      let mut row = model_value(job);
      row["link_id"] = json!(link.id);
      row["ci_name"] = json!(link.name);
      row["ci_stage"] = json!(link.stage);
      row["ci_needs"] = json!(link.needs.unwrap_or_default());
      row["ci_image"] = json!(link.image.unwrap_or_default());
      row["ci_script"] = json!(link.script);
      row["ci_artifacts"] = json!(link.artifacts.unwrap_or_default());
      row["ci_order"] = json!(link.sort_order);
      row
    })
    .collect::<Vec<_>>();
  Ok(json!(
        rows.into_iter()
            .map(|row| {
                let data = payload(&row).unwrap_or(json!({}));
                json!({
                        "pipeline_job":{
                            "id":row["link_id"],
                            "project_id":p,
                            "pipeline_id":i,
                            "project_job_id":row["id"],
                            "name":row["ci_name"],
                            "stage":row["ci_stage"],
                            "image":row["ci_image"],
                            "needs":row["ci_needs"],
                            "script":row["ci_script"],
                            "artifacts":row["ci_artifacts"],
                            "sort_order":row["ci_order"]
                        }
                        ,
                        "status":if row["status"]=="pending"&&text(&row,"run_after").starts_with(&worker::BLOCKED[..10]){json!("blocked")}else{row["status"].clone()},
                        "needs":data["needs"],
                        "script":data["script"],
                        "artifacts":data["artifacts"],
                        "tags":data["tags"],
                        "project_job":public_job(row)
                    }
                )
            })
            .collect::<Vec<_>>()
    ))
}
