use crate::{ApiConfig, CreatesResult};
use az_derive_aliases::{apply, plain_clone_debug};

pub use az_maven::MavenArtifact;

#[apply(plain_clone_debug)]
pub struct MavenCentralApi {
    inner: az_maven::MavenCentralApi,
}

impl MavenCentralApi {
    pub fn new(config: ApiConfig) -> CreatesResult<Self> {
        Ok(Self {
            inner: az_maven::MavenCentralApi::new(config)?,
        })
    }

    pub fn search_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_group_id(group_id, rows)?)
    }

    pub fn search_by_artifact_id(
        &self,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_artifact_id(artifact_id, rows)?)
    }

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

    pub fn search_by_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_class_name(class_name, rows)?)
    }

    pub fn search_by_fully_qualified_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self
            .inner
            .search_by_fully_qualified_class_name(class_name, rows)?)
    }

    pub fn search_by_sha1(
        &self,
        sha1: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_sha1(sha1, rows)?)
    }

    pub fn search_by_tag(
        &self,
        tag: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_tag(tag, rows)?)
    }

    pub fn search_by_keyword(
        &self,
        keyword: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Vec<MavenArtifact>> {
        Ok(self.inner.search_by_keyword(keyword, rows)?)
    }

    pub fn get_latest_version(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
    ) -> CreatesResult<Option<String>> {
        Ok(self.inner.get_latest_version(group_id, artifact_id)?)
    }

    pub fn get_latest_version_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> CreatesResult<Option<String>> {
        Ok(self.inner.get_latest_version_by_group_id(group_id, rows)?)
    }

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

pub fn create_maven_central_api() -> CreatesResult<MavenCentralApi> {
    let config = ApiConfig::builder("https://search.maven.org").build()?;
    MavenCentralApi::new(config)
}
