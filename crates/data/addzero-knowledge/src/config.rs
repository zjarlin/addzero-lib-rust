use std::{
    env,
    path::{Path, PathBuf},
};

use addzero_persistence as persistence;
use deunicode::deunicode;

use crate::types::KnowledgeSourceSpec;

const DEFAULT_RUST_ROOT: &str = "/Users/zjarlin/Desktop/tech-content-automation/rust/sources";
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
    let Some(home) = home_dir() else {
        return specs;
    };

    let rust_root = env::var("DIOXUS_ADMIN_KB_SOURCE_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_RUST_ROOT));
    specs.push(KnowledgeSourceSpec::new("rust", "rust", rust_root));

    specs.push(KnowledgeSourceSpec::new(
        "notes",
        "笔记",
        home.join("Nextcloud/一些未整理的资料/Documents"),
    ));
    specs.push(KnowledgeSourceSpec::new(
        "software-docs",
        "软件文档",
        home.join("Nextcloud/软件文档"),
    ));
    specs.push(KnowledgeSourceSpec::new(
        "memory",
        "memory",
        home.join("memory"),
    ));
    specs.push(KnowledgeSourceSpec::new(
        "docker-compose",
        "docker-compose",
        home.join("Nextcloud/DockerCompose"),
    ));
    specs.push(KnowledgeSourceSpec::new(
        "docker-compose-unused",
        "docker-compose-unused",
        home.join("Nextcloud/DockerCompose_Unuse"),
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
        "requirements",
        "要件",
        home.join("Nextcloud/要件"),
    ));
    specs.push(KnowledgeSourceSpec::new(
        "config-sys",
        "config-sys",
        home.join("Music/addzero/config-sys"),
    ));

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
    env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
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
