// 制品存储抽象：版本发布时把外部制品镜像到本地文件或云对象存储（S3 兼容 / MinIO），
// 返回快的可下载地址（避免直接依赖 GitHub 等慢源分发）。

use std::{fs, path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::config::{CenterConfig, ObjectStorageConfig};

#[async_trait]
pub trait ArtifactStore: Send + Sync + std::fmt::Debug {
    /// 存储制品字节，返回可下载 URL。
    async fn store(
        &self,
        component: &str,
        version: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<String, String>;
}

/// 本地文件存储：写 `{artifact_dir}/{component}/{version}/{filename}`，
/// 下载地址由 center 的制品下载服务提供。
#[derive(Debug)]
pub struct LocalArtifactStore {
    dir: PathBuf,
    public_url: String,
}

impl LocalArtifactStore {
    pub fn new(dir: PathBuf, public_url: &str) -> Self {
        Self {
            dir,
            public_url: public_url.to_string(),
        }
    }
}

#[async_trait]
impl ArtifactStore for LocalArtifactStore {
    async fn store(
        &self,
        component: &str,
        version: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<String, String> {
        let dir = self.dir.join(component).join(version);
        fs::create_dir_all(&dir).map_err(|err| format!("create artifact dir failed: {err}"))?;
        fs::write(dir.join(filename), &bytes)
            .map_err(|err| format!("write artifact failed: {err}"))?;
        Ok(format!(
            "{}/api/v1/releases/artifact/{component}/{version}/{filename}",
            self.public_url.trim_end_matches('/')
        ))
    }
}

/// 云对象存储（S3 兼容 / MinIO）：上传 `{bucket}/{component}/{version}/{filename}`，
/// 返回对象存储直接 URL。
#[derive(Debug)]
pub struct ObjectStorageArtifactStore {
    endpoint: String,
    bucket: String,
    client: aws_sdk_s3::Client,
}

impl ObjectStorageArtifactStore {
    pub fn new(config: &ObjectStorageConfig) -> Result<Self, String> {
        use aws_sdk_s3::config::{Credentials, Region};
        use aws_sdk_s3::Config;

        let creds = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "artifact-store",
        );
        let cfg = Config::builder()
            .endpoint_url(&config.endpoint)
            .region(Region::new("us-east-1"))
            .credentials_provider(creds)
            .build();
        let client = aws_sdk_s3::Client::from_conf(cfg);
        Ok(Self {
            endpoint: config.endpoint.clone(),
            bucket: config.bucket.clone(),
            client,
        })
    }
}

#[async_trait]
impl ArtifactStore for ObjectStorageArtifactStore {
    async fn store(
        &self,
        component: &str,
        version: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<String, String> {
        use aws_sdk_s3::primitives::ByteStream;

        let key = format!("{component}/{version}/{filename}");
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|err| format!("object storage put failed: {err}"))?;
        Ok(format!(
            "{}/{}/{}",
            self.endpoint.trim_end_matches('/'),
            self.bucket,
            key
        ))
    }
}

/// 按配置选择制品存储：对象存储初始化失败时回退本地文件。
pub fn build_artifact_store(config: &CenterConfig) -> Arc<dyn ArtifactStore> {
    if let Some(object_storage) = &config.object_storage {
        if let Ok(store) = ObjectStorageArtifactStore::new(object_storage) {
            return Arc::new(store);
        }
    }
    Arc::new(LocalArtifactStore::new(
        config.artifact_dir.clone(),
        &config.public_url,
    ))
}
