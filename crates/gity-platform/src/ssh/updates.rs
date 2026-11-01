use tokio::io::{AsyncRead, AsyncReadExt};

pub(super) async fn read(
  reader: &mut (impl AsyncRead + Unpin),
) -> anyhow::Result<(Vec<u8>, Vec<(String, String)>)> {
  let mut prefix = Vec::new();
  let mut updates = Vec::new();
  loop {
    let mut len = [0; 4];
    reader.read_exact(&mut len).await?;
    prefix.extend(len);
    if &len == b"0000" {
      break;
    }
    let size = usize::from_str_radix(std::str::from_utf8(&len)?, 16)?;
    anyhow::ensure!(
      (4..=65520).contains(&size) && prefix.len() + size <= 1024 * 1024,
      "Git update command limit exceeded"
    );
    let mut packet = vec![0; size - 4];
    reader.read_exact(&mut packet).await?;
    prefix.extend(&packet);
    let command = std::str::from_utf8(&packet)?
      .split('\0')
      .next()
      .unwrap_or_default();
    let parts = command.split_whitespace().collect::<Vec<_>>();
    anyhow::ensure!(parts.len() == 3, "invalid Git update command");
    updates.push((parts[2].to_owned(), parts[1].to_owned()));
  }
  Ok((prefix, updates))
}
