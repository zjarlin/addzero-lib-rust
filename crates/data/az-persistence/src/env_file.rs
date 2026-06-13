use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const WORKSPACE_ENV_FILE: &str = ".env";
pub(crate) const LOCAL_ENV_FILE: &str = ".config/aio/aio.env";

pub(crate) fn read_database_url_from_path(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let vars = parse_env_pairs(&content);
    vars.get("MSC_AIO_DATABASE_URL")
        .or_else(|| vars.get("DATABASE_URL"))
        .cloned()
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn workspace_env_path_from(start_dir: &Path) -> Option<PathBuf> {
    start_dir
        .ancestors()
        .map(|dir| dir.join(WORKSPACE_ENV_FILE))
        .find(|path| path.is_file())
}

fn parse_env_pairs(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{read_database_url_from_path, workspace_env_path_from};

    #[test]
    fn workspace_env_path_finds_ancestor_env() {
        let root = unique_temp_dir("workspace-env-path");
        let nested = root.join("apps/aio/backend");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join(".env"), "DATABASE_URL=postgresql://root\n").unwrap();

        let found = workspace_env_path_from(&nested).unwrap();
        assert_eq!(found, root.join(".env"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn read_database_url_prefers_msc_key_from_env_file() {
        let dir = unique_temp_dir("workspace-env-read");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        fs::write(
            &path,
            "# comment\nDATABASE_URL=postgresql://fallback\nMSC_AIO_DATABASE_URL=postgresql://preferred\n",
        )
        .unwrap();

        let value = read_database_url_from_path(&path).unwrap();
        assert_eq!(value, "postgresql://preferred");

        fs::remove_dir_all(dir).unwrap();
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("az-persistence-{prefix}-{unique}"))
    }
}
