use crate::configuration::cfg::{Config, IssueAttachments, S3IssueAttachments};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

#[derive(Clone)]
pub struct IssueAttachmentUploadResult {
  pub url: String,
  pub object_key: String,
  pub content_type: String,
  pub size: usize,
}

#[derive(Clone)]
pub struct IssueAttachmentService {
  backend: Option<IssueAttachmentBackend>,
  pub max_file_size: usize,
}

#[derive(Clone)]
enum IssueAttachmentBackend {
  S3(S3IssueAttachmentBackend),
}

#[derive(Clone)]
struct S3IssueAttachmentBackend {
  client: Client,
  bucket: String,
  region: String,
  endpoint: Option<String>,
  public_base_url: Option<String>,
  force_path_style: bool,
}

impl IssueAttachmentService {
  pub fn new(config: &Config) -> Self {
    let issue_attachments = config.issue_attachments.as_ref();
    let max_file_size = issue_attachments
      .and_then(|cfg| cfg.max_file_size)
      .unwrap_or(10 * 1024 * 1024);
    let backend = parse_backend(issue_attachments);
    Self {
      backend,
      max_file_size,
    }
  }

  pub fn is_enabled(&self) -> bool {
    self.backend.is_some()
  }

  pub async fn upload(
    &self,
    object_key: String,
    bytes: Vec<u8>,
    content_type: Option<&str>,
  ) -> Result<IssueAttachmentUploadResult, String> {
    let backend = self
      .backend
      .as_ref()
      .ok_or_else(|| "issue attachment storage is not configured".to_string())?;
    if bytes.is_empty() {
      return Err("attachment is empty".to_string());
    }
    if bytes.len() > self.max_file_size {
      return Err(format!(
        "attachment exceeds max_file_size ({} bytes)",
        self.max_file_size
      ));
    }

    match backend {
      IssueAttachmentBackend::S3(s3) => {
        let normalized_content_type = content_type
          .map(str::trim)
          .filter(|value| !value.is_empty())
          .unwrap_or("application/octet-stream")
          .to_string();

        s3.client
          .put_object()
          .bucket(s3.bucket.as_str())
          .key(object_key.as_str())
          .content_type(normalized_content_type.as_str())
          .body(ByteStream::from(bytes.clone()))
          .send()
          .await
          .map_err(|err| format!("failed to upload attachment to s3: {err}"))?;

        Ok(IssueAttachmentUploadResult {
          url: s3.public_url(object_key.as_str()),
          object_key,
          content_type: normalized_content_type,
          size: bytes.len(),
        })
      }
    }
  }
}

impl S3IssueAttachmentBackend {
  fn public_url(&self, object_key: &str) -> String {
    if let Some(base) = self.public_base_url.as_deref() {
      let base = base.trim_end_matches('/');
      return format!("{base}/{object_key}");
    }
    if let Some(endpoint) = self.endpoint.as_deref() {
      let endpoint = endpoint.trim_end_matches('/');
      if self.force_path_style {
        return format!("{endpoint}/{}/{}", self.bucket, object_key);
      }
      return format!("{endpoint}/{object_key}");
    }
    format!(
      "https://{}.s3.{}.amazonaws.com/{}",
      self.bucket, self.region, object_key
    )
  }
}

fn parse_backend(issue_attachments: Option<&IssueAttachments>) -> Option<IssueAttachmentBackend> {
  let attachments = issue_attachments?;
  let provider = attachments
    .provider
    .as_deref()
    .unwrap_or("s3")
    .trim()
    .to_ascii_lowercase();
  if provider != "s3" {
    return None;
  }

  let s3 = attachments.s3.as_ref()?;
  let bucket = get_required(s3, |cfg| cfg.bucket.as_deref())?;
  let access_key = get_required(s3, |cfg| cfg.access_key.as_deref())?;
  let secret_key = get_required(s3, |cfg| cfg.secret_key.as_deref())?;
  let region = s3
    .region
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .unwrap_or("us-east-1")
    .to_string();
  let endpoint = s3
    .endpoint
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToString::to_string);
  let public_base_url = s3
    .public_base_url
    .as_deref()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToString::to_string);
  let force_path_style = s3.force_path_style.unwrap_or(true);

  let credentials = Credentials::new(access_key, secret_key, None, None, "gity-config");
  let mut builder = S3ConfigBuilder::new()
    .region(Region::new(region.clone()))
    .credentials_provider(credentials)
    .force_path_style(force_path_style);
  if let Some(endpoint) = endpoint.as_deref() {
    builder = builder.endpoint_url(endpoint);
  }
  let client = Client::from_conf(builder.build());

  Some(IssueAttachmentBackend::S3(S3IssueAttachmentBackend {
    client,
    bucket,
    region,
    endpoint,
    public_base_url,
    force_path_style,
  }))
}

fn get_required(
  cfg: &S3IssueAttachments,
  getter: impl Fn(&S3IssueAttachments) -> Option<&str>,
) -> Option<String> {
  getter(cfg)
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(ToString::to_string)
}
