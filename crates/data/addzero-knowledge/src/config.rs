use std::{
    env,
    path::{Path, PathBuf},
};

use addzero_persistence as persistence;
use deunicode::deunicode;

use crate::types::KnowledgeSourceSpec;

pub fn database_url() -> Option<String> {
    persistence::database_url()
}

pub fn local_env_path() -> Option<PathBuf> {
    persistence::local_env_path()
}

pub fn source_specs() -> Vec<KnowledgeSourceSpec> {
    let mut specs = default_source_specs();
    specs.extend(extra_source_specs_from_env());
    specs.sort_by(|left, right| left.name.cmp(&right.name));
    specs.dedup_by(|left, right| left.root_path == right.root_path);
    specs
}

fn default_source_specs() -> Vec<KnowledgeSourceSpec> {
    let mut specs = Vec::new();

    if let Some(rust_root) = env::var("DIOXUS_ADMIN_KB_SOURCE_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
    {
        specs.push(KnowledgeSourceSpec::new("rust", "rust", rust_root));
    }

    if let Some(home) = home_dir() {
        specs.push(KnowledgeSourceSpec::new(
            "memory",
            "memory",
            home.join("memory"),
        ));
        specs.push(KnowledgeSourceSpec::new(
            "shell-config",
            "shell",
            home.join(".config/shell"),
        ));
        specs.push(KnowledgeSourceSpec::new(
            "mole-config",
            "mole",
            home.join(".config/mole"),
        ));
        specs.push(KnowledgeSourceSpec::new(
            "config-sys",
            "config-sys",
            home.join("Music/addzero/config-sys"),
        ));
    }

    specs
        .into_iter()
        .filter(|spec| spec.root_path.exists())
        .collect()
}

fn extra_source_specs_from_env() -> Vec<KnowledgeSourceSpec> {
    let raw = env::var("MSC_AIO_KNOWLEDGE_EXTRA_ROOTS")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let Some(raw) = raw else {
        return Vec::new();
    };

    raw.split(['\n', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| {
            let path = PathBuf::from(value);
            if !path.exists() {
                return None;
            }

            let name = path
                .file_name()
                .map(|item| item.to_string_lossy().to_string())
                .unwrap_or_else(|| value.to_string());
            let slug = slugify(&name);
            Some(KnowledgeSourceSpec::new(slug, name, path))
        })
        .collect()
}

fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

fn slugify(value: &str) -> String {
    let normalized = deunicode(value);
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in normalized.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

#[allow(dead_code)]
fn _assert_path(_: &Path) {}
