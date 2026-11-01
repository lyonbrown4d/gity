use super::*;
use axum::{
  http::{HeaderValue, header},
  response::{IntoResponse, Response},
};
pub(super) async fn metadata_xml(
  s: &AppState,
  project_id: i64,
  path: &str,
) -> Result<String, AppError> {
  let (group, artifact, snapshot) = coordinates(path)?;
  let name = format!("{group}:{artifact}");
  let detail = named(s, project_id, "maven", &name).await?;
  let versions = detail["versions"].as_array().cloned().unwrap_or_default();
  Ok(if let Some(snapshot_version) = snapshot {
    snapshot_metadata(&group, &artifact, &snapshot_version, &versions)
  } else {
    artifact_metadata(&group, &artifact, &versions)
  })
}

pub(super) fn xml_response(body: String) -> Response {
  let mut response = body.into_response();
  response.headers_mut().insert(
    header::CONTENT_TYPE,
    HeaderValue::from_static("application/xml; charset=utf-8"),
  );
  response
}

pub(in crate::registry) fn validate_metadata_document(bytes: &[u8]) -> Result<(), AppError> {
  let source = std::str::from_utf8(bytes)
    .map_err(|_| AppError::bad_request("Maven metadata must be UTF-8 XML"))?;
  let mut reader = quick_xml::Reader::from_str(source);
  let mut depth = 0usize;
  loop {
    match reader.read_event() {
      Ok(quick_xml::events::Event::Start(_)) => depth += 1,
      Ok(quick_xml::events::Event::Empty(_)) => {}
      Ok(quick_xml::events::Event::End(_)) if depth > 0 => depth -= 1,
      Ok(quick_xml::events::Event::End(_)) => {
        return Err(AppError::bad_request("invalid Maven metadata XML"));
      }
      Ok(quick_xml::events::Event::Eof) if depth == 0 => return Ok(()),
      Ok(quick_xml::events::Event::Eof) => {
        return Err(AppError::bad_request("invalid Maven metadata XML"));
      }
      Ok(_) => {}
      Err(_) => return Err(AppError::bad_request("invalid Maven metadata XML")),
    }
  }
}

pub(super) fn coordinates(path: &str) -> Result<(String, String, Option<String>), AppError> {
  let parts = path.split('/').collect::<Vec<_>>();
  if parts.len() < 3 || parts.last() != Some(&"maven-metadata.xml") {
    return Err(AppError::bad_request("invalid Maven metadata path"));
  }
  let tail = parts.len() - 2;
  let snapshot = parts[tail]
    .ends_with("-SNAPSHOT")
    .then(|| parts[tail].to_owned());
  let artifact_index = if snapshot.is_some() { tail - 1 } else { tail };
  if artifact_index == 0 {
    return Err(AppError::bad_request("invalid Maven metadata coordinates"));
  }
  let artifact = parts[artifact_index].to_owned();
  let group = parts[..artifact_index].join(".");
  if group.is_empty() || artifact.is_empty() {
    return Err(AppError::bad_request("invalid Maven metadata coordinates"));
  }
  Ok((group, artifact, snapshot))
}

pub(super) fn artifact_metadata(group: &str, artifact: &str, versions: &[Value]) -> String {
  let mut values = versions
    .iter()
    .filter_map(|entry| entry["version"].get("version")?.as_str().map(str::to_owned))
    .collect::<Vec<_>>();
  values.sort_by(compare_versions);
  values.dedup();
  let latest = values.last().cloned().unwrap_or_default();
  let release = values
    .iter()
    .rev()
    .find(|version| !version.ends_with("-SNAPSHOT"))
    .cloned()
    .unwrap_or_default();
  let versions_xml = values
    .iter()
    .map(|version| format!("<version>{}</version>", escape(version)))
    .collect::<String>();
  format!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata><groupId>{}</groupId><artifactId>{}</artifactId><versioning><latest>{}</latest><release>{}</release><versions>{}</versions><lastUpdated>{}</lastUpdated></versioning></metadata>\n",
    escape(group),
    escape(artifact),
    escape(&latest),
    escape(&release),
    versions_xml,
    latest_updated(versions)
  )
}

fn latest_updated(versions: &[Value]) -> String {
  versions
    .iter()
    .filter_map(|entry| metadata_time(text(&entry["version"], "updated_at")))
    .max()
    .unwrap_or_else(|| "19700101000000".to_owned())
}

fn metadata_time(value: &str) -> Option<String> {
  chrono::DateTime::parse_from_rfc3339(value)
    .map(|timestamp| timestamp.format("%Y%m%d%H%M%S").to_string())
    .or_else(|_| {
      chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .map(|timestamp| timestamp.format("%Y%m%d%H%M%S").to_string())
    })
    .ok()
}

fn compare_versions(left: &String, right: &String) -> std::cmp::Ordering {
  let parse = |value: &str| {
    let mut parts = value.split('.').collect::<Vec<_>>();
    if parts.len() == 2 {
      parts.push("0");
    }
    semver::Version::parse(&parts.join(".")).ok()
  };
  match (parse(left), parse(right)) {
    (Some(left), Some(right)) => left.cmp(&right),
    _ => left.cmp(right),
  }
}

pub(super) fn snapshot_metadata(
  group: &str,
  artifact: &str,
  version: &str,
  versions: &[Value],
) -> String {
  let entry = versions
    .iter()
    .find(|entry| text(&entry["version"], "version") == version);
  let files = entry
    .and_then(|entry| entry["files"].as_array())
    .cloned()
    .unwrap_or_default();
  let mut snapshot_versions =
    std::collections::BTreeMap::<(String, String), (String, u64, String, String)>::new();
  let mut latest = None::<(String, u64)>;
  for file in &files {
    let name = text(file, "file_name");
    if [".sha1", ".sha256", ".md5"]
      .iter()
      .any(|v| name.ends_with(v))
    {
      continue;
    }
    let Some(remainder) = name.strip_prefix(&format!("{artifact}-")) else {
      continue;
    };
    let Some((stem, extension)) = remainder.rsplit_once('.') else {
      continue;
    };
    let parts = stem.split('-').collect::<Vec<_>>();
    let Some(index) = parts.iter().position(|part| {
      part.len() == 15
        && part.as_bytes().get(8) == Some(&b'.')
        && part
          .bytes()
          .enumerate()
          .all(|(i, byte)| i == 8 || byte.is_ascii_digit())
    }) else {
      continue;
    };
    let Some(candidate_build) = parts
      .get(index + 1)
      .filter(|part| part.chars().all(|v| v.is_ascii_digit()))
    else {
      continue;
    };
    let prefix = parts[..index].join("-");
    let Some(snapshot_base) = version.strip_suffix("-SNAPSHOT") else {
      continue;
    };
    if prefix != snapshot_base || index + 1 >= parts.len() {
      continue;
    }
    let candidate_build = candidate_build.parse::<u64>().unwrap_or_default();
    let candidate_timestamp = parts[index].to_owned();
    let classifier = parts[index + 2..].join("-");
    let key = (extension.to_owned(), classifier);
    let version_updated = entry
      .and_then(|entry| metadata_time(text(&entry["version"], "updated_at")))
      .unwrap_or_else(|| "19700101000000".to_owned());
    let file_updated = metadata_time(text(file, "updated_at")).unwrap_or(version_updated);
    let candidate = (
      candidate_timestamp.clone(),
      candidate_build,
      snapshot_base.to_owned(),
      file_updated,
    );
    if snapshot_versions
      .get(&key)
      .is_none_or(|current| (&candidate.0, candidate.1) > (&current.0, current.1))
    {
      snapshot_versions.insert(key, candidate);
    }
    if latest
      .as_ref()
      .is_none_or(|(time, number)| (&candidate_timestamp, candidate_build) > (time, *number))
    {
      latest = Some((candidate_timestamp, candidate_build));
    }
  }
  let snapshot = latest
    .as_ref()
    .map_or_else(String::new, |(timestamp, build)| {
      format!(
        "<snapshot><timestamp>{}</timestamp><buildNumber>{}</buildNumber></snapshot>",
        escape(timestamp),
        build
      )
    });
  let snapshot_version_entries = snapshot_versions
    .iter()
    .map(|((extension, classifier), (timestamp, build, base, updated))| {
      let classifier_xml = if classifier.is_empty() {
        String::new()
      } else {
        format!("<classifier>{}</classifier>", escape(classifier))
      };
      format!(
        "<snapshotVersion><extension>{}</extension>{}<value>{}-{}-{}</value><updated>{}</updated></snapshotVersion>",
        escape(extension), classifier_xml, escape(base), escape(timestamp), build, updated
      )
    })
    .collect::<String>();
  format!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata><groupId>{}</groupId><artifactId>{}</artifactId><version>{}</version><versioning>{}<lastUpdated>{}</lastUpdated><snapshotVersions>{}</snapshotVersions></versioning></metadata>\n",
    escape(group),
    escape(artifact),
    escape(version),
    snapshot,
    entry
      .and_then(|entry| metadata_time(text(&entry["version"], "updated_at")))
      .unwrap_or_else(|| "19700101000000".to_owned()),
    snapshot_version_entries
  )
}

fn escape(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&apos;")
}
