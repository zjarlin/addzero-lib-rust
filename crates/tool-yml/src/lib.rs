use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YmlError {
    #[error("failed to read yaml file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse yaml file at {path}: {source}")]
    ParseFile {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("yaml path is invalid: {0}")]
    InvalidPath(String),
    #[error("failed to read current directory: {0}")]
    CurrentDir(#[source] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlPathSegment {
    Key(String),
    Index(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YamlPath {
    segments: Vec<YamlPathSegment>,
}

impl YamlPath {
    pub fn parse(path: impl AsRef<str>) -> Result<Self, YmlError> {
        let input = path.as_ref().trim();
        if input.is_empty() {
            return Err(YmlError::InvalidPath("path cannot be empty".to_owned()));
        }

        let chars: Vec<char> = input.chars().collect();
        let mut index = 0usize;
        let mut segments = Vec::new();

        while index < chars.len() {
            skip_whitespace(&chars, &mut index);

            if index < chars.len() && chars[index] == '.' {
                index += 1;
                skip_whitespace(&chars, &mut index);
                if index >= chars.len() {
                    return Err(YmlError::InvalidPath(format!(
                        "path `{input}` cannot end with `.`"
                    )));
                }
            }

            if index >= chars.len() {
                break;
            }

            if chars[index] == '[' {
                segments.push(parse_bracket_segment(&chars, &mut index, input)?);
            } else {
                segments.push(parse_bare_segment(&chars, &mut index, input)?);
            }

            skip_whitespace(&chars, &mut index);
            if index < chars.len() && chars[index] != '.' && chars[index] != '[' {
                return Err(YmlError::InvalidPath(format!(
                    "unexpected character `{}` in path `{input}`",
                    chars[index]
                )));
            }
        }

        Ok(Self { segments })
    }

    pub fn segments(&self) -> &[YamlPathSegment] {
        &self.segments
    }
}

impl FromStr for YamlPath {
    type Err = YmlError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct YamlDoc {
    value: Value,
}

impl YamlDoc {
    pub fn from_value(value: Value) -> Self {
        Self { value }
    }

    pub fn as_value(&self) -> &Value {
        &self.value
    }

    pub fn into_inner(self) -> Value {
        self.value
    }

    pub fn get_path(&self, path: &YamlPath) -> Option<&Value> {
        lookup_value(&self.value, path)
    }

    pub fn get(&self, path: &str) -> Result<Option<&Value>, YmlError> {
        let parsed = YamlPath::parse(path)?;
        Ok(self.get_path(&parsed))
    }

    pub fn get_string(&self, path: &str) -> Result<Option<String>, YmlError> {
        let parsed = YamlPath::parse(path)?;
        Ok(self.get_string_at(&parsed))
    }

    pub fn get_string_at(&self, path: &YamlPath) -> Option<String> {
        self.get_path(path)
            .and_then(stringify_scalar)
            .map(|value| env_subst(&value))
    }
}

pub trait YamlLookup {
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value>;
}

impl YamlLookup for YamlDoc {
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value> {
        self.get_path(path)
    }
}

impl YamlLookup for Value {
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value> {
        lookup_value(self, path)
    }
}

impl<T> YamlLookup for &T
where
    T: YamlLookup + ?Sized,
{
    fn yaml_lookup(&self, path: &YamlPath) -> Option<&Value> {
        (*self).yaml_lookup(path)
    }
}

pub fn get_yaml_path_value<'a, T>(doc: &'a T, path: &YamlPath) -> Option<&'a Value>
where
    T: YamlLookup + ?Sized,
{
    doc.yaml_lookup(path)
}

pub fn load_yaml<T, P>(path: P) -> Result<T, YmlError>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|source| YmlError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    serde_yaml::from_str::<T>(&content).map_err(|source| YmlError::ParseFile {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load_yaml_value<P>(path: P) -> Result<YamlDoc, YmlError>
where
    P: AsRef<Path>,
{
    load_yaml::<Value, _>(path).map(YamlDoc::from_value)
}

pub fn env_subst(input: impl AsRef<str>) -> String {
    let source = input.as_ref();
    let mut result = String::with_capacity(source.len());
    let mut cursor = 0usize;

    while let Some(relative_start) = source[cursor..].find("${") {
        let start = cursor + relative_start;
        result.push_str(&source[cursor..start]);

        let placeholder = &source[start + 2..];
        if let Some(relative_end) = placeholder.find('}') {
            let end = start + 2 + relative_end;
            let body = &source[start + 2..end];
            let (name, default_value) = body.split_once(':').unwrap_or((body, ""));
            let value = env::var(name)
                .ok()
                .filter(|candidate| !candidate.trim().is_empty())
                .unwrap_or_else(|| default_value.to_owned());
            result.push_str(&value);
            cursor = end + 1;
        } else {
            result.push_str(&source[start..]);
            cursor = source.len();
            break;
        }
    }

    if cursor < source.len() {
        result.push_str(&source[cursor..]);
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpringYaml {
    root: PathBuf,
}

impl SpringYaml {
    pub fn from_dir(path: impl Into<PathBuf>) -> Self {
        Self { root: path.into() }
    }

    pub fn from_current_dir() -> Result<Self, YmlError> {
        let root = env::current_dir().map_err(YmlError::CurrentDir)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve_resource(&self, resource_name: &str) -> PathBuf {
        let base_name = resource_name
            .strip_suffix(".yml")
            .or_else(|| resource_name.strip_suffix(".yaml"))
            .unwrap_or(resource_name);

        let extensions = if resource_name.contains('.') {
            ["", ".yml", ".yaml"]
        } else {
            [".yml", ".yaml", ""]
        };

        for extension in extensions {
            if extension.is_empty() && !resource_name.contains('.') {
                continue;
            }

            let candidate = self.root.join(format!("{base_name}{extension}"));
            if candidate.exists() {
                return candidate;
            }
        }

        let fallback_extension = if resource_name.contains('.') {
            ""
        } else {
            ".yml"
        };
        self.root.join(format!("{base_name}{fallback_extension}"))
    }

    pub fn get_yml_content(&self, resource_name: &str) -> Result<String, YmlError> {
        let path = self.resolve_resource(resource_name);
        fs::read_to_string(&path).map_err(|source| YmlError::ReadFile { path, source })
    }

    pub fn load_named(&self, resource_name: &str) -> Result<YamlDoc, YmlError> {
        let path = self.resolve_resource(resource_name);
        load_yaml_value(path)
    }

    pub fn load_active(&self) -> Result<YamlDoc, YmlError> {
        let primary = self.load_named("application")?;
        let profile = primary
            .get_string("spring.profiles.active")?
            .filter(|value| !value.trim().is_empty());

        if let Some(profile_name) = profile {
            let active_path = self.resolve_resource(&format!("application-{profile_name}"));
            if active_path.exists() {
                return load_yaml_value(active_path);
            }
        }

        Ok(primary)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub jdbc_url: String,
    pub jdbc_username: Option<String>,
    pub jdbc_password: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DatabaseConfigReader;

impl DatabaseConfigReader {
    pub fn read(
        path: impl AsRef<Path>,
        prefer_data_source_name: Option<&str>,
    ) -> Result<Option<DatabaseConfig>, YmlError> {
        let spring_yaml = SpringYaml::from_dir(path.as_ref().to_path_buf());
        let active = spring_yaml.load_active()?;
        Self::read_from_doc(&active, prefer_data_source_name)
    }

    pub fn read_from_doc(
        doc: &YamlDoc,
        prefer_data_source_name: Option<&str>,
    ) -> Result<Option<DatabaseConfig>, YmlError> {
        if let Some(name) = prefer_data_source_name {
            if let Some(config) = Self::read_named_data_source(doc, name)? {
                return Ok(Some(config));
            }
        }

        for url_path in SINGLE_DATASOURCE_PATHS {
            if let Some(url) = read_non_blank_property(doc, url_path)? {
                let base_path = extract_base_path(url_path);
                let username = read_non_blank_property(doc, &format!("{base_path}.username"))?
                    .or(read_non_blank_property(doc, "spring.datasource.username")?)
                    .or(read_non_blank_property(doc, "spring.r2dbc.username")?);
                let password = read_non_blank_property(doc, &format!("{base_path}.password"))?
                    .or(read_non_blank_property(doc, "spring.datasource.password")?)
                    .or(read_non_blank_property(doc, "spring.r2dbc.password")?);

                return Ok(Some(DatabaseConfig {
                    jdbc_url: url,
                    jdbc_username: username,
                    jdbc_password: password,
                }));
            }
        }

        for data_source_name in COMMON_DATA_SOURCE_NAMES {
            if let Some(config) = Self::read_named_data_source(doc, data_source_name)? {
                return Ok(Some(config));
            }
        }

        Ok(None)
    }

    fn read_named_data_source(
        doc: &YamlDoc,
        data_source_name: &str,
    ) -> Result<Option<DatabaseConfig>, YmlError> {
        let url_paths = [
            format!("spring.datasource.{data_source_name}.url"),
            format!("spring.datasource.{data_source_name}.jdbc-url"),
            format!("spring.datasource.dynamic.datasource.{data_source_name}.url"),
            format!("spring.datasource.mp.datasource.{data_source_name}.url"),
        ];

        for url_path in url_paths {
            if let Some(url) = read_non_blank_property(doc, &url_path)? {
                let base_path = extract_base_path(&url_path);
                let username = read_non_blank_property(doc, &format!("{base_path}.username"))?;
                let password = read_non_blank_property(doc, &format!("{base_path}.password"))?;

                return Ok(Some(DatabaseConfig {
                    jdbc_url: url,
                    jdbc_username: username,
                    jdbc_password: password,
                }));
            }
        }

        Ok(None)
    }
}

const SINGLE_DATASOURCE_PATHS: &[&str] = &[
    "spring.datasource.url",
    "spring.datasource.jdbc-url",
    "spring.r2dbc.url",
    "spring.datasource.primary.url",
    "spring.datasource.master.url",
    "spring.datasource.default.url",
    "spring.data.jdbc.url",
];

const COMMON_DATA_SOURCE_NAMES: &[&str] = &["master", "primary", "default", "main", "slave"];

fn read_non_blank_property(doc: &YamlDoc, path: &str) -> Result<Option<String>, YmlError> {
    Ok(doc
        .get_string(path)?
        .filter(|value| !value.trim().is_empty()))
}

fn extract_base_path(url_path: &str) -> String {
    url_path
        .rsplit_once('.')
        .map(|(base, _)| {
            base.trim_end_matches(".jdbc")
                .trim_end_matches(".r2dbc")
                .to_owned()
        })
        .unwrap_or_else(|| url_path.to_owned())
}

fn skip_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}

fn parse_bare_segment(
    chars: &[char],
    index: &mut usize,
    original: &str,
) -> Result<YamlPathSegment, YmlError> {
    let mut segment = String::new();
    while *index < chars.len() && chars[*index] != '.' && chars[*index] != '[' {
        segment.push(chars[*index]);
        *index += 1;
    }

    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return Err(YmlError::InvalidPath(format!(
            "empty segment in path `{original}`"
        )));
    }

    Ok(YamlPathSegment::Key(trimmed.to_owned()))
}

fn parse_bracket_segment(
    chars: &[char],
    index: &mut usize,
    original: &str,
) -> Result<YamlPathSegment, YmlError> {
    *index += 1;
    skip_whitespace(chars, index);

    if *index >= chars.len() {
        return Err(YmlError::InvalidPath(format!(
            "unclosed bracket in path `{original}`"
        )));
    }

    let segment = if matches!(chars[*index], '"' | '\'') {
        let quote = chars[*index];
        *index += 1;
        let mut value = String::new();
        let mut closed = false;

        while *index < chars.len() {
            let current = chars[*index];
            if current == '\\' {
                *index += 1;
                if *index < chars.len() {
                    value.push(chars[*index]);
                    *index += 1;
                }
                continue;
            }

            if current == quote {
                *index += 1;
                closed = true;
                break;
            }

            value.push(current);
            *index += 1;
        }

        if !closed {
            return Err(YmlError::InvalidPath(format!(
                "unclosed quoted segment in path `{original}`"
            )));
        }

        YamlPathSegment::Key(value)
    } else {
        let mut raw = String::new();
        while *index < chars.len() && chars[*index] != ']' {
            raw.push(chars[*index]);
            *index += 1;
        }

        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(YmlError::InvalidPath(format!(
                "empty bracket segment in path `{original}`"
            )));
        }

        if trimmed.chars().all(|character| character.is_ascii_digit()) {
            let value = trimmed.parse::<usize>().map_err(|_| {
                YmlError::InvalidPath(format!(
                    "invalid sequence index `{trimmed}` in `{original}`"
                ))
            })?;
            YamlPathSegment::Index(value)
        } else {
            YamlPathSegment::Key(trimmed.to_owned())
        }
    };

    skip_whitespace(chars, index);
    if *index >= chars.len() || chars[*index] != ']' {
        return Err(YmlError::InvalidPath(format!(
            "missing closing `]` in path `{original}`"
        )));
    }
    *index += 1;

    Ok(segment)
}

fn lookup_value<'a>(root: &'a Value, path: &YamlPath) -> Option<&'a Value> {
    let mut current = root;

    for segment in path.segments() {
        current = match segment {
            YamlPathSegment::Key(key) => {
                let mapping = current.as_mapping()?;
                let key = Value::String(key.clone());
                mapping.get(&key)?
            }
            YamlPathSegment::Index(index) => current.as_sequence()?.get(*index)?,
        };
    }

    Some(current)
}

fn stringify_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(inner) => Some(inner.clone()),
        Value::Number(inner) => Some(inner.to_string()),
        Value::Bool(inner) => Some(inner.to_string()),
        _ => None,
    }
}

#[macro_export]
macro_rules! yaml_path {
    ($path:literal) => {{
        <$crate::YamlPath as ::std::str::FromStr>::from_str($path)
            .expect("yaml_path!: invalid path literal")
    }};
    ($($path:tt)+) => {{
        <$crate::YamlPath as ::std::str::FromStr>::from_str(::core::stringify!($($path)+))
            .expect("yaml_path!: invalid path tokens")
    }};
}

#[macro_export]
macro_rules! yaml_get {
    ($doc:expr, $path:literal) => {{
        let __path = $crate::yaml_path!($path);
        $crate::get_yaml_path_value(&$doc, &__path)
    }};
    ($doc:expr, $($path:tt)+) => {{
        let __path = $crate::yaml_path!($($path)+);
        $crate::get_yaml_path_value(&$doc, &__path)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use tempfile::TempDir;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct AppConfig {
        spring: SpringSection,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct SpringSection {
        application: ApplicationSection,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct ApplicationSection {
        name: String,
    }

    #[test]
    fn load_yaml_reads_typed_value() {
        let temp = TempDir::new().expect("temp dir should be created");
        let path = temp.path().join("application.yml");
        fs::write(&path, "spring:\n  application:\n    name: addzero-lib\n")
            .expect("yaml should be written");

        let parsed = load_yaml::<AppConfig, _>(&path).expect("yaml should deserialize");

        assert_eq!(
            parsed,
            AppConfig {
                spring: SpringSection {
                    application: ApplicationSection {
                        name: "addzero-lib".to_owned(),
                    },
                },
            }
        );
    }

    #[test]
    fn yaml_path_macro_supports_bracket_segments() {
        let doc = YamlDoc::from_value(
            serde_yaml::from_str(
                "spring:\n  datasource:\n    jdbc-url: jdbc:mysql://localhost:3306/app\n",
            )
            .expect("yaml should parse"),
        );

        let path = yaml_path!(spring.datasource["jdbc-url"]);
        let value = doc
            .get_path(&path)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let macro_value = yaml_get!(doc, spring.datasource["jdbc-url"])
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        assert_eq!(value.as_deref(), Some("jdbc:mysql://localhost:3306/app"));
        assert_eq!(macro_value, value);
    }

    #[test]
    fn env_subst_uses_env_and_defaults() {
        let expanded = env_subst("${CARGO_MANIFEST_DIR}");
        let defaulted = env_subst("jdbc:${ADDZERO_MISSING_TEST_VAR:sqlite}");
        let empty = env_subst("${ADDZERO_MISSING_TEST_VAR}");

        assert!(!expanded.is_empty());
        assert_eq!(defaulted, "jdbc:sqlite");
        assert_eq!(empty, "");
    }

    #[test]
    fn resolve_resource_prefers_yml_and_falls_back_to_requested_name() {
        let temp = TempDir::new().expect("temp dir should be created");
        fs::write(temp.path().join("config.yml"), "yml: content\n").expect("yml should exist");
        fs::write(temp.path().join("config.yaml"), "yaml: content\n").expect("yaml should exist");
        fs::write(temp.path().join("other.yaml"), "yaml: content\n").expect("yaml should exist");

        let spring_yaml = SpringYaml::from_dir(temp.path());

        assert_eq!(
            spring_yaml.resolve_resource("config"),
            temp.path().join("config.yml")
        );
        assert_eq!(
            spring_yaml.resolve_resource("other.yaml"),
            temp.path().join("other.yaml")
        );
        assert_eq!(
            spring_yaml.resolve_resource("missing.txt"),
            temp.path().join("missing.txt")
        );
    }

    #[test]
    fn load_active_uses_profile_file_and_falls_back_to_primary() {
        let temp = TempDir::new().expect("temp dir should be created");
        fs::write(
            temp.path().join("application.yml"),
            "spring:\n  profiles:\n    active: dev\nfeature:\n  enabled: false\n",
        )
        .expect("application.yml should be written");
        fs::write(
            temp.path().join("application-dev.yaml"),
            "feature:\n  enabled: true\n",
        )
        .expect("profile file should be written");

        let active = SpringYaml::from_dir(temp.path())
            .load_active()
            .expect("active file should load");
        let enabled = active
            .get("feature.enabled")
            .expect("path should parse")
            .and_then(Value::as_bool);

        assert_eq!(enabled, Some(true));

        fs::remove_file(temp.path().join("application-dev.yaml"))
            .expect("profile file should be removed");
        let fallback = SpringYaml::from_dir(temp.path())
            .load_active()
            .expect("fallback file should load");
        let fallback_enabled = fallback
            .get("feature.enabled")
            .expect("path should parse")
            .and_then(Value::as_bool);
        assert_eq!(fallback_enabled, Some(false));
    }

    #[test]
    fn database_config_reader_handles_supported_layouts() {
        let temp = TempDir::new().expect("temp dir should be created");
        fs::write(
            temp.path().join("application.yml"),
            "spring:\n  datasource:\n    url: jdbc:mysql://localhost:3306/app\n    username: root\n    password: secret\n",
        )
        .expect("application should be written");

        let single = DatabaseConfigReader::read(temp.path(), None)
            .expect("config should load")
            .expect("config should exist");
        assert_eq!(single.jdbc_url, "jdbc:mysql://localhost:3306/app");
        assert_eq!(single.jdbc_username.as_deref(), Some("root"));
        assert_eq!(single.jdbc_password.as_deref(), Some("secret"));

        fs::write(
            temp.path().join("application.yml"),
            "spring:\n  datasource:\n    master:\n      url: jdbc:mysql://localhost:3306/master\n      username: root\n      password: master-pass\n    slave:\n      url: jdbc:mysql://localhost:3306/slave\n      username: slave-user\n      password: slave-pass\n",
        )
        .expect("application should be overwritten");

        let preferred = DatabaseConfigReader::read(temp.path(), Some("slave"))
            .expect("config should load")
            .expect("preferred config should exist");
        assert_eq!(preferred.jdbc_url, "jdbc:mysql://localhost:3306/slave");
        assert_eq!(preferred.jdbc_username.as_deref(), Some("slave-user"));
        assert_eq!(preferred.jdbc_password.as_deref(), Some("slave-pass"));

        let fallback = DatabaseConfigReader::read(temp.path(), None)
            .expect("config should load")
            .expect("fallback config should exist");
        assert_eq!(fallback.jdbc_url, "jdbc:mysql://localhost:3306/master");
    }

    #[test]
    fn database_config_reader_returns_none_when_missing() {
        let temp = TempDir::new().expect("temp dir should be created");
        fs::write(temp.path().join("application.yml"), "app:\n  name: demo\n")
            .expect("application should be written");

        let config = DatabaseConfigReader::read(temp.path(), None).expect("config should load");

        assert_eq!(config, None);
    }
}
