use std::io;
use std::path::Path;

/// Simple support routines for the Git smart HTTP protocol.
///
/// The advertisement step can be implemented entirely with `git2` by listing
/// the repository references and emitting them in pkt-line format.  This keeps
/// us free of any dependency on the `git` binary.

/// Build the `info/refs` response for a repository.  `service` should be
/// either `"upload-pack"` or `"receive-pack"` (without the `git-` prefix).
pub fn advertise_refs<P: AsRef<Path>>(repo_path: P, service: &str) -> io::Result<Vec<u8>> {
  let repo = git2::Repository::open_bare(repo_path.as_ref())
    .map_err(|err| io::Error::other(err.to_string()))?;
  let refs = repo
    .references()
    .map_err(|err| io::Error::other(err.to_string()))?;

  fn pkt_line(s: &str) -> Vec<u8> {
    let len = s.len() + 4;
    format!("{:04x}{}", len, s).into_bytes()
  }

  let mut buf = Vec::new();
  // service announcement
  buf.extend(pkt_line(&format!("# service=git-{}\n", service)));
  buf.extend(b"0000");

  for item in refs {
    let reference = item.map_err(|err| io::Error::other(err.to_string()))?;
    let Some(id) = reference.target() else {
      continue;
    };
    let Some(name) = reference.name() else {
      continue;
    };
    let line = format!("{id} {name}\n");
    buf.extend(pkt_line(&line));
  }
  buf.extend(b"0000");
  Ok(buf)
}
