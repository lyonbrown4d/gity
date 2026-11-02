pub(crate) fn safe_after_login_redirect(path: &str) -> Option<String> {
  let (route, query) = path.split_once('?').unwrap_or((path, ""));
  if route != "/oauth/consent" {
    return None;
  }
  let request_id = query
    .split('&')
    .filter_map(|part| part.split_once('='))
    .find_map(|(key, value)| (key == "request_id").then_some(value))?;
  if request_id.len() != 36
    || !request_id.bytes().enumerate().all(|(i, byte)| {
      if [8, 13, 18, 23].contains(&i) {
        byte == b'-'
      } else {
        byte.is_ascii_hexdigit()
      }
    })
  {
    return None;
  }
  Some(format!("/oauth/consent?request_id={request_id}"))
}

#[cfg(test)]
mod tests;
