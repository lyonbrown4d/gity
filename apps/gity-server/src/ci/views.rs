use super::*;
pub(crate) async fn pipeline_detail(s: &AppState, p: i64, i: i64) -> Result<Value, AppError> {
    let row = one(s, "project_pipelines", p, i).await?;
    let spec = config::parse(text(&row, "config_content"))?;
    Ok(json!({"pipeline":row,"spec":spec,"jobs":pipeline_jobs(s,p,i).await?}))
}
pub(crate) async fn pipeline_jobs(s: &AppState, p: i64, i: i64) -> Result<Value, AppError> {
    let rows=s.db.all("SELECT l.id AS link_id,l.name AS ci_name,l.stage AS ci_stage,l.needs AS ci_needs,l.image AS ci_image,l.script AS ci_script,l.artifacts AS ci_artifacts,l.sort_order AS ci_order,j.* FROM project_pipeline_jobs l JOIN project_jobs j ON j.id=l.project_job_id WHERE l.project_id=? AND l.pipeline_id=? ORDER BY l.sort_order",
        vec![p.into(),
            i.into()]).await?;
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
