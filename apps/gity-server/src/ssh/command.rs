use gity_git::GitRpc;

pub(super) fn parse(raw: &[u8]) -> anyhow::Result<(GitRpc, String)> {
    anyhow::ensure!(raw.len() <= 4096, "SSH command is too long");
    let raw = std::str::from_utf8(raw)?;
    let (service, path) = raw
        .split_once(' ')
        .ok_or_else(|| anyhow::anyhow!("only Git upload-pack and receive-pack are supported"))?;
    let service = match service {
        "git-upload-pack" => GitRpc::UploadPack,
        "git-receive-pack" => GitRpc::ReceivePack,
        _ => anyhow::bail!("only Git upload-pack and receive-pack are supported"),
    };
    let path = path
        .strip_prefix('\'')
        .and_then(|s| s.strip_suffix('\''))
        .unwrap_or(path);
    let path = path.strip_prefix('/').unwrap_or(path);
    anyhow::ensure!(
        path.ends_with(".git")
            && path.split('/').count() >= 2
            && path.split('/').all(|v| !v.is_empty()
                && v != "."
                && v != ".."
                && v.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))),
        "invalid SSH repository path"
    );
    Ok((service, path.strip_suffix(".git").unwrap().to_owned()))
}
#[cfg(test)]
#[path = "command_tests.rs"]
mod tests;
