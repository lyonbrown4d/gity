use super::*;

pub(super) async fn rpc(
    s: AppState,
    org: String,
    name: String,
    req: Request,
    write: bool,
) -> Result<Response, AppError> {
    let headers = req.headers().clone();
    let service = if write {
        "git-receive-pack"
    } else {
        "git-upload-pack"
    };
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        != Some(&format!("application/x-{service}-request"))
    {
        return Err(AppError::bad_request("invalid Git content type"));
    }
    let (project, h) = project(&s, &headers, &org, &name, write).await?;
    let path = repository::repo(&project);
    let stream = req.into_body().into_data_stream();
    let mut reader = StreamReader::new(futures_util::TryStreamExt::map_err(
        stream,
        std::io::Error::other,
    ));
    let mut env = BTreeMap::new();
    if let Some(v) = headers.get("git-protocol").and_then(|v| v.to_str().ok()) {
        env.insert("GIT_PROTOCOL".into(), v.into());
    }
    let mut prefix = Vec::new();
    let mut updates = Vec::new();
    if write {
        s.git
            .ensure_update_hook(
                &path,
                s.settings
                    .git_hook
                    .clone()
                    .unwrap_or(std::env::current_exe()?),
            )
            .await
            .map_err(repository::git_error)?;
        let rules =
            s.db.all(
                "SELECT * FROM project_branch_protections WHERE project_id=?",
                vec![crate::projects::id(&project["id"])?.into()],
            )
            .await?;
        let mut deny_force = Vec::new();
        loop {
            let mut len = [0; 4];
            reader.read_exact(&mut len).await?;
            prefix.extend(len);
            if &len == b"0000" {
                break;
            }
            let size = usize::from_str_radix(
                std::str::from_utf8(&len)
                    .map_err(|_| AppError::bad_request("invalid packet length"))?,
                16,
            )
            .map_err(|_| AppError::bad_request("invalid packet length"))?;
            if !(4..=65520).contains(&size) || prefix.len() + size > 1024 * 1024 {
                return Err(AppError::bad_request("Git update command limit exceeded"));
            }
            let mut packet = vec![0; size - 4];
            reader.read_exact(&mut packet).await?;
            prefix.extend(&packet);
            let text = std::str::from_utf8(&packet)
                .map_err(|_| AppError::bad_request("invalid update command"))?
                .split('\0')
                .next()
                .unwrap_or_default();
            let parts: Vec<_> = text.split_whitespace().collect();
            if parts.len() != 3 {
                return Err(AppError::bad_request("invalid update command"));
            }
            let reference = parts[2];
            updates.push((reference.to_owned(), parts[1].to_owned()));
            if let Some(branch) = reference.strip_prefix("refs/heads/") {
                repository::check_branch(
                    &s,
                    &h,
                    &project,
                    branch,
                    parts[1].bytes().all(|b| b == b'0'),
                    false,
                )
                .await?;
                if branch == project["default_branch"].as_str().unwrap_or("main")
                    && parts[1].bytes().all(|b| b == b'0')
                {
                    return Err(AppError::forbidden("cannot delete default branch"));
                }
                if let Some(rule) = repository::effective_rule(&rules, branch)
                    && !rule["allow_force_push"]
                        .as_bool()
                        .unwrap_or_else(|| rule["allow_force_push"].as_i64() == Some(1))
                {
                    deny_force.push(reference.to_owned());
                }
            } else if !reference.starts_with("refs/tags/") {
                return Err(AppError::forbidden("reference namespace not writable"));
            }
        }
        env.insert("GITY_DENY_FORCE_PUSH_REFS".into(), deny_force.join("\n"));
    }
    let mut process = s
        .git
        .spawn_rpc(
            &path,
            if write {
                GitRpc::ReceivePack
            } else {
                GitRpc::UploadPack
            },
            false,
            env,
        )
        .await
        .map_err(repository::git_error)?;
    let mut stdin = process
        .take_stdin()
        .ok_or_else(|| AppError::internal("Git input missing"))?;
    tokio::spawn(async move {
        let result = async {
            stdin.write_all(&prefix).await?;
            tokio::io::copy(&mut reader, &mut stdin).await?;
            stdin.shutdown().await
        }
        .await;
        if let Err(e) = result {
            tracing::debug!(error=%e,"Git request stream ended");
        }
    });
    let body = Body::from_stream(ReaderStream::new(
        process
            .into_reader_with_completion(move || async move {
                for (reference, sha) in updates {
                    // receive-pack may exit successfully even when an individual ref
                    // was rejected. Trigger work only for a confirmed, accepted tip.
                    if sha.bytes().all(|v| v == b'0') {
                        continue;
                    }
                    let current = s
                        .git
                        .resolve_revision(&path, &reference, repository::default_branch(&project))
                        .await;
                    if current.as_ref().is_ok_and(|value| value == &sha) {
                        crate::events::repository_changed(&s, &project, &reference, &sha).await;
                    }
                }
            })
            .map_err(repository::git_error)?,
    ));
    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                format!("application/x-{service}-result"),
            ),
            (header::CACHE_CONTROL, "no-cache".into()),
        ],
        body,
    )
        .into_response())
}
