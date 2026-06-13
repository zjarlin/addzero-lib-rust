use crate::http::HttpApiClient;
use crate::util::non_blank;
use crate::config::ApiConfig;
use az_derive_aliases::{apply, deserialize_debug, impl_from_match, plain_clone_debug, serde_eq};

#[apply(serde_eq)]
pub struct MavenArtifact {
    pub id: String,
    pub group_id: String,
    pub artifact_id: String,
    pub latest_version: Option<String>,
    pub version: Option<String>,
    pub packaging: Option<String>,
    pub timestamp: Option<i64>,
}

impl MavenArtifact {
    pub fn resolved_version(&self) -> Option<&str> {
        self.version.as_deref().or(self.latest_version.as_deref())
    }
}

#[apply(plain_clone_debug)]
pub struct MavenCentralApi {
    http: HttpApiClient,
}

impl MavenCentralApi {
    pub fn new(config: ApiConfig) -> anyhow::Result<Self> {
        Ok(Self {
            http: HttpApiClient::new(config)?,
        })
    }

    pub fn search_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(format!("g:{}", group_id.as_ref().trim()), rows, None)
    }

    pub fn search_by_artifact_id(
        &self,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(format!("a:{}", artifact_id.as_ref().trim()), rows, None)
    }

    pub fn search_by_coordinates(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        let query = format!(
            "g:{} AND a:{}",
            group_id.as_ref().trim(),
            artifact_id.as_ref().trim()
        );
        self.search(query, rows, None)
    }

    pub fn search_all_versions(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        let query = format!(
            "g:{} AND a:{}",
            group_id.as_ref().trim(),
            artifact_id.as_ref().trim()
        );
        self.search(query, rows, Some("gav"))
    }

    pub fn search_by_full_coordinates(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        version: Option<&str>,
        packaging: Option<&str>,
        classifier: Option<&str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        let mut conditions = vec![
            format!("g:{}", group_id.as_ref().trim()),
            format!("a:{}", artifact_id.as_ref().trim()),
        ];

        if let Some(value) = non_blank(version) {
            conditions.push(format!("v:{value}"));
        }
        if let Some(value) = non_blank(packaging) {
            conditions.push(format!("p:{value}"));
        }
        if let Some(value) = non_blank(classifier) {
            conditions.push(format!("l:{value}"));
        }

        self.search(conditions.join(" AND "), rows, None)
    }

    pub fn search_by_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(format!("c:{}", class_name.as_ref().trim()), rows, None)
    }

    pub fn search_by_fully_qualified_class_name(
        &self,
        class_name: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(format!("fc:{}", class_name.as_ref().trim()), rows, None)
    }

    pub fn search_by_sha1(
        &self,
        sha1: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(format!("1:{}", sha1.as_ref().trim()), rows, None)
    }

    pub fn search_by_tag(
        &self,
        tag: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(format!("tags:{}", tag.as_ref().trim()), rows, None)
    }

    pub fn search_by_keyword(
        &self,
        keyword: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        self.search(keyword.as_ref().trim().to_owned(), rows, None)
    }

    pub fn get_latest_version(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
    ) -> anyhow::Result<Option<String>> {
        let artifacts = self.search_by_coordinates(group_id, artifact_id, 1)?;
        Ok(artifacts
            .first()
            .and_then(|artifact| artifact.latest_version.clone().or(artifact.version.clone())))
    }

    pub fn get_latest_version_by_group_id(
        &self,
        group_id: impl AsRef<str>,
        rows: usize,
    ) -> anyhow::Result<Option<String>> {
        let artifacts = self.search_by_group_id(group_id, rows)?;
        Ok(artifacts
            .first()
            .and_then(|artifact| artifact.latest_version.clone().or(artifact.version.clone())))
    }

    pub fn download_file(
        &self,
        group_id: impl AsRef<str>,
        artifact_id: impl AsRef<str>,
        version: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> anyhow::Result<Vec<u8>> {
        let filepath = format!(
            "{}/{}/{}/{}",
            group_id.as_ref().replace('.', "/"),
            artifact_id.as_ref().trim(),
            version.as_ref().trim(),
            filename.as_ref().trim()
        );

        let response = self
            .http
            .get("/remotecontent")?
            .query(&[("filepath", filepath)])
            .send()?;
        HttpApiClient::read_bytes(response)
    }

    fn search(
        &self,
        query: String,
        rows: usize,
        core: Option<&str>,
    ) -> anyhow::Result<Vec<MavenArtifact>> {
        let mut params = vec![
            ("q", query),
            ("rows", rows.max(1).to_string()),
            ("wt", "json".to_owned()),
        ];

        if let Some(value) = core {
            params.push(("core", value.to_owned()));
        }

        let response = self.http.get("/solrsearch/select")?.query(&params).send()?;
        let response: MavenSearchResponseEnvelope = HttpApiClient::read_json(response)?;

        Ok(response
            .response
            .docs
            .into_iter()
            .map(MavenArtifact::from)
            .collect())
    }
}

pub fn create_maven_central_api() -> anyhow::Result<MavenCentralApi> {
    let config = ApiConfig::builder("https://search.maven.org").build()?;
    MavenCentralApi::new(config)
}

#[apply(deserialize_debug)]
struct MavenSearchResponseEnvelope {
    response: MavenSearchResponse,
}

#[apply(deserialize_debug)]
struct MavenSearchResponse {
    #[serde(default)]
    docs: Vec<MavenSearchDocument>,
}

#[apply(deserialize_debug)]
struct MavenSearchDocument {
    #[serde(default)]
    id: String,
    #[serde(rename = "g", default)]
    group_id: String,
    #[serde(rename = "a", default)]
    artifact_id: String,
    #[serde(rename = "latestVersion", default)]
    latest_version: Option<String>,
    #[serde(rename = "v", default)]
    version: Option<String>,
    #[serde(rename = "p", default)]
    packaging: Option<String>,
    #[serde(default)]
    timestamp: Option<i64>,
}

impl_from_match!(MavenSearchDocument => MavenArtifact {
    value => MavenArtifact {
        id: value.id,
        group_id: value.group_id,
        artifact_id: value.artifact_id,
        latest_version: value.latest_version,
        version: value.version,
        packaging: value.packaging,
        timestamp: value.timestamp,
    }
});
