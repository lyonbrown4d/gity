use super::{Repository, request};
use anyhow::{Result, bail};
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn cleanup(repository: &Repository) -> Result<()> {
    let Some(resources) = &repository.cleanup else {
        return Ok(());
    };
    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    let token_result = request(
        &client,
        &resources.base_url,
        "DELETE",
        &format!(
            "/projects/{}/access-tokens/{}",
            resources.project_id, resources.access_token_id
        ),
        Some(&resources.api_token),
        None,
    );
    let project_result = request(
        &client,
        &resources.base_url,
        "DELETE",
        &format!("/projects/{}", resources.project_id),
        Some(&resources.api_token),
        Some(json!({"confirmation": resources.project_path})),
    );
    let session_result = request(
        &client,
        &resources.base_url,
        "POST",
        "/auth/logout",
        Some(&resources.api_token),
        Some(json!({"refresh_token": ""})),
    );
    let failures = [
        token_result
            .err()
            .map(|error| format!("token revocation: {error}")),
        project_result
            .err()
            .map(|error| format!("project deletion request: {error}")),
        session_result
            .err()
            .map(|error| format!("session logout: {error}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if failures.is_empty() {
        println!(
            "Marked stress project for deletion and revoked its access token: user_id={} organization_id={} project_id={} token_id={}; the account and organization remain for test-database lifecycle cleanup.",
            resources.user_id,
            resources.organization_id,
            resources.project_id,
            resources.access_token_id
        );
        Ok(())
    } else {
        bail!(
            "stress resource cleanup incomplete: {}",
            failures.join("; ")
        )
    }
}
