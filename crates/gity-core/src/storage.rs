use crate::AppError;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use object_store::{
    ObjectStore, ObjectStoreExt, aws::AmazonS3Builder, local::LocalFileSystem,
    path::Path as ObjectPath,
};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};
#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Storage {
    inner: Arc<dyn ObjectStore>,
    local_root: Option<PathBuf>,
}
pub struct StorageMeta {
    pub size: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub etag: Option<String>,
}
impl Storage {
    pub fn local(root: impl AsRef<Path>) -> Result<Self, AppError> {
        std::fs::create_dir_all(root.as_ref())?;
        let root = root.as_ref().canonicalize()?;
        let inner = LocalFileSystem::new_with_prefix(&root).map_err(storage_error)?;
        Ok(Self {
            inner: Arc::new(inner),
            local_root: Some(root),
        })
    }
    pub fn from_settings(settings: &crate::Settings) -> Result<Self, AppError> {
        let root = &settings.storage_root;
        let config = &settings.storage;
        match config.driver.as_str() {
            "local" => Self::local(root),
            "s3" => {
                let mut builder = AmazonS3Builder::from_env()
                    .with_bucket_name(config.s3_bucket.clone())
                    .with_region(config.s3_region.clone())
                    .with_virtual_hosted_style_request(!config.s3_use_path_style);
                let endpoint = config.s3_endpoint.clone();
                if !endpoint.is_empty() {
                    builder = builder
                        .with_allow_http(endpoint.starts_with("http://"))
                        .with_endpoint(endpoint);
                }
                let key = config.s3_access_key.clone();
                if !key.is_empty() {
                    builder = builder
                        .with_access_key_id(key)
                        .with_secret_access_key(config.s3_secret_key.clone());
                }
                Ok(Self {
                    inner: Arc::new(builder.build().map_err(storage_error)?),
                    local_root: None,
                })
            }
            _ => Err(AppError::bad_request("unsupported storage driver")),
        }
    }
    fn path(&self, key: &str) -> Result<ObjectPath, AppError> {
        if key.is_empty()
            || key.contains('\\')
            || key.contains('\0')
            || Path::new(key)
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(AppError::bad_request("invalid storage key"));
        }
        if let Some(root) = &self.local_root {
            let path = root.join(key);
            let mut parent = path.as_path();
            while !parent.exists() {
                if std::fs::symlink_metadata(parent).is_ok() {
                    return Err(AppError::bad_request("invalid storage path"));
                }
                parent = parent
                    .parent()
                    .ok_or_else(|| AppError::bad_request("invalid storage path"))?;
            }
            if !parent.canonicalize()?.starts_with(root) {
                return Err(AppError::bad_request("storage path escapes root"));
            }
        }
        ObjectPath::parse(key).map_err(|_| AppError::bad_request("invalid storage key"))
    }
    pub async fn put(&self, key: &str, bytes: Bytes) -> Result<(), AppError> {
        let path = self.path(key)?;
        self.inner
            .put(&path, bytes.into())
            .await
            .map_err(storage_error)?;
        Ok(())
    }
    pub async fn get(&self, key: &str) -> Result<Bytes, AppError> {
        self.inner
            .get(&self.path(key)?)
            .await
            .map_err(storage_error)?
            .bytes()
            .await
            .map_err(storage_error)
    }
    pub async fn head(&self, key: &str) -> Result<StorageMeta, AppError> {
        let m = self
            .inner
            .head(&self.path(key)?)
            .await
            .map_err(storage_error)?;
        Ok(StorageMeta {
            size: m.size,
            last_modified: m.last_modified,
            etag: m.e_tag,
        })
    }
    pub async fn get_range(
        &self,
        key: &str,
        range: std::ops::Range<u64>,
    ) -> Result<Bytes, AppError> {
        self.inner
            .get_range(&self.path(key)?, range)
            .await
            .map_err(storage_error)
    }
    pub async fn exists(&self, key: &str) -> Result<bool, AppError> {
        match self.inner.head(&self.path(key)?).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(e) => Err(storage_error(e)),
        }
    }
    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.inner
            .delete(&self.path(key)?)
            .await
            .map_err(storage_error)
    }
    pub async fn stream(
        &self,
        key: &str,
    ) -> Result<impl Stream<Item = Result<Bytes, AppError>> + Send + 'static, AppError> {
        Ok(self
            .inner
            .get(&self.path(key)?)
            .await
            .map_err(storage_error)?
            .into_stream()
            .map(|r| r.map_err(storage_error)))
    }
    pub async fn put_stream<S>(&self, key: &str, stream: S, maximum: u64) -> Result<u64, AppError>
    where
        S: Stream<Item = Result<Bytes, AppError>> + Send,
    {
        let mut upload = self
            .inner
            .put_multipart(&self.path(key)?)
            .await
            .map_err(storage_error)?;
        futures_util::pin_mut!(stream);
        let mut total = 0;
        let mut pending = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(v) => v,
                Err(e) => {
                    let _ = upload.abort().await;
                    return Err(e);
                }
            };
            total += chunk.len() as u64;
            if total > maximum {
                let _ = upload.abort().await;
                return Err(AppError::bad_request("upload size limit exceeded"));
            }
            pending.extend_from_slice(&chunk);
            if pending.len() >= 8 * 1024 * 1024
                && let Err(e) = upload
                    .put_part(Bytes::from(std::mem::take(&mut pending)).into())
                    .await
            {
                let _ = upload.abort().await;
                return Err(storage_error(e));
            }
        }
        if (!pending.is_empty() || total == 0)
            && let Err(e) = upload.put_part(Bytes::from(pending).into()).await
        {
            let _ = upload.abort().await;
            return Err(storage_error(e));
        }
        upload.complete().await.map_err(storage_error)?;
        Ok(total)
    }
}
fn storage_error(error: object_store::Error) -> AppError {
    if matches!(error, object_store::Error::NotFound { .. }) {
        return AppError::not_found("stored object not found");
    }
    tracing::error!(error=%error,"object storage operation failed");
    AppError::internal("object storage operation failed")
}
