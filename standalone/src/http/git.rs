use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use crate::http::app_state::AppState;
use git::GitRepository;
use serde::Deserialize;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use git::http; // import http helper from git crate

#[derive(Deserialize)]
pub struct InfoRefsParams {
    service: String,
}

/// GET /git/:owner/:repo/info/refs?service=git-<name>
pub async fn info_refs(
    Path((owner, repo)): Path<(String, String)>,
    Query(params): Query<InfoRefsParams>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    // determine repository path from configuration
    // let root = match &app_state.config.storage {
    //     Some(s) => &s.repo_root,
    //     None => return (StatusCode::INTERNAL_SERVER_ERROR, "storage.repo_root not set").into_response(),
    // };
    // let repo_name = repo.strip_suffix(".git").unwrap_or(&repo);
    // let repo_path = std::path::Path::new(root)
    //     .join(owner)
    //     .join(format!("{}.git", repo_name));
    //
    // // make sure repo exists; we don't actually need to open with gix for
    // // advertisement since the CLI will do the work, but we can verify it.
    // if !repo_path.exists() {
    //     return (StatusCode::NOT_FOUND, "repository not found").into_response();
    // }
    // // strip leading "git-" from service
    // let svc = params.service.strip_prefix("git-").unwrap_or(&params.service);
    // match http::advertise_refs(&repo_path, svc) {
    //     Ok(body) => {
    //         let mut headers = HeaderMap::new();
    //         let content_type = format!("application/x-git-{}-advertisement", svc);
    //         headers.insert("Content-Type", content_type.parse().unwrap());
    //         (StatusCode::OK, headers, body).into_response()
    //     }
    //     Err(err) => {
    //         (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             format!("error preparing refs: {}", err),
    //         )
    //             .into_response()
    //     }
    // }
}

// POST /git/:owner/:repo/<service>  where service is git-upload-pack or git-receive-pack
// pub async fn service_pack(
//     Path((owner, repo, service)): Path<(String, String, String)>,
//     State(app_state): State<AppState>,
//     body: Bytes,
// ) -> Response {
//     // // map service name to git subcommand
//     // let git_service = match service.as_str() {
//     //     "git-upload-pack" | "git-receive-pack" => service.as_str(),
//     //     _ => return (StatusCode::BAD_REQUEST, "unknown service").into_response(),
//     // };
//     //
//     // let root = match &app_state.config.storage {
//     //     Some(s) => &s.repo_root,
//     //     None => return (StatusCode::INTERNAL_SERVER_ERROR, "storage.repo_root not set").into_response(),
//     // };
//     // let repo_name = repo.strip_suffix(".git").unwrap_or(&repo);
//     // let repo_path = std::path::Path::new(root)
//     //     .join(&owner)
//     //     .join(format!("{}.git", repo_name));
//     //
//     // // validate repository via our `git` crate (which wraps gix)
//     // if let Err(e) = GitRepository::open(&repo_path) {
//     //     eprintln!("failed to open repo: {}", e);
//     //     return (StatusCode::NOT_FOUND, "repository not found").into_response();
//     // }
//     //
//     // // spawn git process (temporary fallback)
//     // let mut child = match Command::new("git")
//     //     .arg("-C")
//     //     .arg(&repo_path)
//     //     .arg(git_service)
//     //     .arg("--stateless-rpc")
//     //     .arg(".")
//     //     .stdin(std::process::Stdio::piped())
//     //     .stdout(std::process::Stdio::piped())
//     //     .spawn()
//     // {
//     //     Ok(c) => c,
//     //     Err(err) => {
//     //         eprintln!("failed to spawn git: {}", err);
//     //         return (StatusCode::INTERNAL_SERVER_ERROR, "git spawn failed").into_response();
//     //     }
//     // };
//     //
//     // // write request body to git stdin
//     // if let Some(mut stdin) = child.stdin.take() {
//     //     if let Err(e) = stdin.write_all(&body).await {
//     //         eprintln!("failed to write to git stdin: {}", e);
//     //     }
//     // }
//     //
//     // // collect output
//     // let output = match child.wait_with_output().await {
//     //     Ok(o) => o,
//     //     Err(e) => {
//     //         eprintln!("git process error: {}", e);
//     //         return (StatusCode::INTERNAL_SERVER_ERROR, "git process failed").into_response();
//     //     }
//     // };
//     //
//     // // return git stdout as response (protocol already sets packet-lines etc)
//     // (StatusCode::OK, output.stdout).into_response()
// }

// return a router for git service endpoints
// pub fn git_routes() -> axum::Router<AppState> {
//     use axum::routing::{get, post};
//     axum::Router::new()
//         .route(
//             "/{owner}/{repo}/info/refs",
//             get(info_refs),
//         )
//         .route(
//             "/{owner}/{repo}/{service}",
//             post(service_pack),
//         )
// }
