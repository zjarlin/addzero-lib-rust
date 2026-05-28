use crate::{ApiConfig, CreatesResult};
use az_derive_aliases::{apply, plain_clone_debug};

/// Maven Central 构件搜索结果模型，直接复用 `az-maven` 的 wire DTO。
pub use az_maven::MavenArtifact;

/// Maven Central 客户端门面。
///
/// 该类型只负责把 `az-maven` 的错误映射到 [`CreatesError`](crate::CreatesError)，
/// 并保留 `az-creates` 统一配置入口，不重新定义 Maven 查询协议。
#[apply(plain_clone_debug)]
pub struct MavenCentralApi {
    inner: az_maven::MavenCentralApi,
}

impl MavenCentralApi {
    /// 使用显式 API 配置创建 Maven Central 客户端。
    pub fn new(config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            inner: az_maven::MavenCentralApi::new(config)?,
        })
    }

    /// 按 `groupId` 查询构件，`rows` 控制返回数量上限。
    pub fn search_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_group_id(group_id, rows)?)
    }

    /// 按 `artifactId` 查询构件，适合不知道 group 的模糊检索。
    pub fn search_by_artifact_id(
        &self,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_artifact_id(artifact_id, rows)?)
    }

    /// 按 `groupId + artifactId` 查询构件的最新搜索结果。
    pub fn search_by_coordinates(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self
            .inner
            .search_by_coordinates(group_id, artifact_id, rows)?)
    }

    /// 查询指定坐标的全部版本记录。
    pub fn search_all_versions(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self
            .inner
            .search_all_versions(group_id, artifact_id, rows)?)
    }

    /// 按完整 Maven 坐标查询，支持版本、打包类型和 classifier 过滤。
    pub fn search_by_full_coordinates(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        version: Option<&str>,
        packaging: Option<&str>,
        classifier: Option<&str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_full_coordinates(
            group_id,
            artifact_id,
            version,
            packaging,
            classifier,
            rows,
        )?)
    }

    /// 按类名查询包含该类的构件。
    pub fn search_by_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_class_name(class_name, rows)?)
    }

    /// 按完整限定类名查询包含该类的构件。
    pub fn search_by_fully_qualified_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self
            .inner
            .search_by_fully_qualified_class_name(class_name, rows)?)
    }

    /// 按文件 SHA-1 查询构件。
    pub fn search_by_sha1(
        &self,
        sha1: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_sha1(sha1, rows)?)
    }

    /// 按 Maven Central tag 查询构件。
    pub fn search_by_tag(
        &self,
        tag: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_tag(tag, rows)?)
    }

    /// 按关键词查询构件。
    pub fn search_by_keyword(
        &self,
        keyword: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_keyword(keyword, rows)?)
    }

    /// 获取指定 `groupId + artifactId` 的最新版本号。
    pub fn get_latest_version(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
    ) -> CreatesResult<Option<String>> {
        Ok(self.inner.get_latest_version(group_id, artifact_id)?)
    }

    /// 获取某个 `groupId` 下搜索结果中的最新版本号。
    pub fn get_latest_version_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Option<String>> {
        Ok(self.inner.get_latest_version_by_group_id(group_id, rows)?)
    }

    /// 通过 Maven Central `remotecontent` 端点下载指定构件文件。
    pub fn download_file(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        version: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> CreatesResult<Vec<u8>> {
        Ok(self
            .inner
            .download_file(group_id, artifact_id, version, filename)?)
    }
}

/// 使用默认 `https://search.maven.org` 地址创建 Maven Central 客户端。
pub fn create_maven_central_api() -> CreatesResult<MavenCentralApi> {
    let config = ApiConfig::builder("https://search.maven.org").build()?;
    MavenCentralApi::new(config)
}
