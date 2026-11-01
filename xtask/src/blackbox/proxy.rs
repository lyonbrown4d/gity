use anyhow::Result;

pub fn read() -> Result<Option<url::Url>> {
  let Some(value) = std::env::var_os("GITY_BLACKBOX_DOWNLOAD_PROXY") else {
    return Ok(None);
  };
  let proxy = url::Url::parse(&value.to_string_lossy())
    .map_err(|_| anyhow::anyhow!("invalid black-box download proxy URL"))?;
  anyhow::ensure!(
    proxy.scheme() == "http"
      && proxy.host_str().is_some()
      && proxy.username().is_empty()
      && proxy.password().is_none()
      && proxy.path() == "/"
      && proxy.query().is_none()
      && proxy.fragment().is_none(),
    "black-box download proxy must be an HTTP proxy without credentials or a path"
  );
  Ok(Some(proxy))
}
