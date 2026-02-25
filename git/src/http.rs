use std::io;
use std::path::Path;

/// Simple support routines for the Git smart HTTP protocol.
///
/// The advertisement step can be implemented entirely with `gix` by listing
/// the repository references and emitting them in pkt-line format.  This keeps
/// us free of any dependency on the `git` binary.

/// Build the `info/refs` response for a repository.  `service` should be
/// either `"upload-pack"` or `"receive-pack"` (without the `git-` prefix).
pub fn advertise_refs<P: AsRef<Path>>(repo_path: P, service: &str) -> io::Result<Vec<u8>> {
    // open repository with gix
    let repo = gix::open(repo_path.as_ref()).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let platform = repo
        .references()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let mut iter = platform
        .all()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fn pkt_line(s: &str) -> Vec<u8> {
        let len = s.len() + 4;
        format!("{:04x}{}", len, s).into_bytes()
    }

    let mut buf = Vec::new();
    // service announcement
    buf.extend(pkt_line(&format!("# service=git-{}\n", service)));
    buf.extend(b"0000");

    while let Some(item) = iter.next() {
        let reference = item.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        if let Some(id) = reference.target().try_id() {
            let line = format!("{} {}\n", id, reference.name());
            buf.extend(pkt_line(&line));
        }
    }
    buf.extend(b"0000");
    Ok(buf)
}
