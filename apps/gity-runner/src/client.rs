use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    base: String,
    token: String,
}
impl Client {
    pub fn new(server: &str, token: String) -> Result<Self> {
        let url = reqwest::Url::parse(server).context("invalid Gity server URL")?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            bail!("Gity server URL must be HTTP(S) without embedded credentials");
        }
        Ok(Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            base: server.trim_end_matches('/').to_owned(),
            token,
        })
    }
    pub async fn post(&self, path: &str, mut body: Value) -> Result<Value> {
        body["token"] = json!(self.token);
        let response = self
            .http
            .post(format!("{}/api/v1{path}", self.base))
            .json(&body)
            .send()
            .await
            .context("contact Gity server")?;
        let status = response.status();
        let value: Value = response
            .json()
            .await
            .context("decode Gity server response")?;
        if !status.is_success() || value["code"] != 0 {
            bail!(
                "Gity request rejected ({status}): {}",
                value["message"].as_str().unwrap_or("unknown error")
            );
        }
        Ok(value["data"].clone())
    }
    pub async fn download(
        &self,
        path: &str,
        mut body: Value,
        destination: &std::path::Path,
        maximum: u64,
    ) -> Result<()> {
        body["token"] = json!(self.token);
        let mut response = self
            .http
            .post(format!("{}/api/v1{path}", self.base))
            .json(&body)
            .send()
            .await
            .context("download job source")?;
        if !response.status().is_success() {
            bail!("source download rejected ({})", response.status());
        }
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)
            .await?;
        use tokio::io::AsyncWriteExt;
        let mut total = 0u64;
        while let Some(chunk) = response.chunk().await? {
            total += chunk.len() as u64;
            if total > maximum {
                bail!("source archive exceeds compressed size limit");
            }
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(())
    }
}
