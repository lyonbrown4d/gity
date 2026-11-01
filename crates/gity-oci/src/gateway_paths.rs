use crate::{MirrorResolveError, OciAuthorizationDecision, OciOperation};
use axum::http::{Method, StatusCode, Uri};

type ParsedRequest = (String, OciOperation, Option<String>, Option<String>);
type ParseError = (StatusCode, &'static str);

pub(super) fn parse_request(method: &Method, uri: &Uri) -> Result<ParsedRequest, ParseError> {
  if uri.path() == "/v2" || uri.path() == "/v2/" {
    if matches!(method, &Method::GET | &Method::HEAD) {
      return Ok((String::new(), OciOperation::Probe, None, None));
    }
    return Err((
      StatusCode::METHOD_NOT_ALLOWED,
      "unsupported OCI probe method",
    ));
  }
  let tail = uri
    .path()
    .strip_prefix("/v2/")
    .ok_or((StatusCode::NOT_FOUND, "invalid OCI path"))?;
  if tail == "_catalog" {
    if !matches!(method, &Method::GET | &Method::HEAD) {
      return Err((StatusCode::METHOD_NOT_ALLOWED, "unsupported catalog method"));
    }
    return Ok((String::new(), OciOperation::Catalog, None, None));
  }
  let mut parts = tail.split('/');
  let mut path_parts = Vec::new();
  let (repository, resource) = loop {
    let part = parts
      .next()
      .ok_or((StatusCode::NOT_FOUND, "invalid OCI path"))?;
    path_parts.push(part);
    if matches!(part, "manifests" | "blobs" | "tags" | "uploads") {
      let consumed = path_parts.len();
      let repository = path_parts[..consumed - 1].join("/");
      let resource = std::iter::once(part)
        .chain(parts)
        .collect::<Vec<_>>()
        .join("/");
      break (repository, resource);
    }
  };
  if !is_repository_path(&repository) {
    return Err((StatusCode::BAD_REQUEST, "invalid repository path"));
  }
  let operation = match (method, resource.split('/').next().unwrap_or_default()) {
    (&Method::GET | &Method::HEAD, "manifests" | "blobs" | "tags") => OciOperation::Pull,
    (&Method::PUT | &Method::PATCH | &Method::POST, "manifests" | "blobs" | "uploads") => {
      OciOperation::Push
    }
    (&Method::DELETE, "manifests" | "blobs") => OciOperation::Delete,
    _ => return Err((StatusCode::METHOD_NOT_ALLOWED, "unsupported OCI operation")),
  };
  let mount_digest = uri.query().and_then(|query| {
    url::form_urlencoded::parse(query.as_bytes())
      .find(|(key, _)| key == "mount")
      .map(|(_, value)| value.into_owned())
  });
  let is_mount =
    method == Method::POST && resource.starts_with("blobs/uploads") && mount_digest.is_some();
  let source = if is_mount {
    uri.query().and_then(|query| {
      url::form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "from")
        .map(|(_, value)| value.into_owned())
    })
  } else {
    None
  };
  if is_mount && source.is_none() {
    return Err((StatusCode::BAD_REQUEST, "mount source required"));
  }
  let reference = resource
    .strip_prefix("manifests/")
    .filter(|reference| !reference.contains('/'))
    .map(str::to_owned);
  Ok((repository.to_owned(), operation, source, reference))
}

pub(super) fn map_mirror_error(error: MirrorResolveError) -> (StatusCode, &'static str) {
  match error {
    MirrorResolveError::Unauthenticated => (
      StatusCode::UNAUTHORIZED,
      "mirror source authentication failed",
    ),
    MirrorResolveError::Forbidden => (StatusCode::FORBIDDEN, "mirror source access denied"),
    MirrorResolveError::Unavailable | MirrorResolveError::InvalidResponse => {
      (StatusCode::BAD_GATEWAY, "mirror source unavailable")
    }
  }
}

pub(super) fn is_sha256_digest(digest: &str) -> bool {
  digest
    .strip_prefix("sha256:")
    .is_some_and(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

pub(super) fn replace_manifest_reference(
  uri: &Uri,
  digest: &str,
) -> Result<Uri, (StatusCode, &'static str)> {
  let (prefix, _) = uri
    .path()
    .rsplit_once("/manifests/")
    .ok_or((StatusCode::BAD_REQUEST, "invalid manifest route"))?;
  let path_and_query = format!(
    "{prefix}/manifests/{digest}{}",
    uri
      .query()
      .map(|query| format!("?{query}"))
      .unwrap_or_default()
  );
  path_and_query
    .parse()
    .map_err(|_| (StatusCode::BAD_REQUEST, "invalid manifest route"))
}

pub(super) fn target_uri(
  upstream: &str,
  uri: &Uri,
  public_repository: &str,
  decision: &OciAuthorizationDecision,
) -> Result<String, (StatusCode, &'static str)> {
  if public_repository.is_empty() {
    return Ok(format!(
      "{}{}{}",
      upstream.trim_end_matches('/'),
      uri.path(),
      uri
        .query()
        .map(|value| format!("?{value}"))
        .unwrap_or_default()
    ));
  }
  let suffix = uri
    .path()
    .strip_prefix("/v2/")
    .and_then(|tail| tail.strip_prefix(public_repository))
    .and_then(|tail| tail.strip_prefix('/'))
    .ok_or((StatusCode::NOT_FOUND, "invalid OCI path"))?;
  let path = format!("/v2/{}/{}", decision.storage_repository, suffix);
  let query = uri.query().map(|query| {
    let pairs = url::form_urlencoded::parse(query.as_bytes()).map(|(key, value)| {
      let value = if key == "from" {
        decision
          .mount_source_storage_repository
          .clone()
          .unwrap_or_else(|| value.into_owned())
      } else {
        value.into_owned()
      };
      (key.into_owned(), value)
    });
    url::form_urlencoded::Serializer::new(String::new())
      .extend_pairs(pairs)
      .finish()
  });
  Ok(format!(
    "{}{}{}",
    upstream.trim_end_matches('/'),
    path,
    query.map(|value| format!("?{value}")).unwrap_or_default()
  ))
}

pub(super) fn validate_storage_path(path: &str) -> Result<(), (StatusCode, &'static str)> {
  if !is_repository_path(path) {
    return Err((
      StatusCode::INTERNAL_SERVER_ERROR,
      "invalid storage repository mapping",
    ));
  }
  Ok(())
}

fn is_repository_path(path: &str) -> bool {
  !path.is_empty()
    && path.split('/').all(|part| {
      !part.is_empty()
        && part != "."
        && part != ".."
        && part.len() <= 255
        && part.bytes().all(|byte| {
          byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    })
}
