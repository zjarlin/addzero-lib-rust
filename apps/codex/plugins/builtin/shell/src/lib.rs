#![forbid(unsafe_code)]

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use codex_plugin_api::{
    CodexPlugin, ContributionSet, GeneratedFileContribution, GeneratedFileStatus, PluginActivation,
    PluginDependency, PluginDescriptor, PluginKind, ShellEntryContribution, ShellEntryKind,
};

const PLUGIN_ID: &str = "builtin/shell";
const DEFAULT_SOURCE_ROOT: &str = ".config/shell";
const DEFAULT_OUTPUT_FILE: &str = ".add_fn";
const SECTION_DELIMITER: &str = "#####";

#[derive(Clone, Debug)]
pub struct ShellPlugin {
    source_root: PathBuf,
    output_path: PathBuf,
    extra_cli_roots: Vec<PathBuf>,
    scan: Option<ShellScan>,
    generated_file: Option<GeneratedFileContribution>,
}

impl ShellPlugin {
    pub fn new(
        source_root: impl Into<PathBuf>,
        output_path: impl Into<PathBuf>,
        extra_cli_roots: Vec<PathBuf>,
    ) -> Self {
        Self {
            source_root: source_root.into(),
            output_path: output_path.into(),
            extra_cli_roots,
            scan: None,
            generated_file: None,
        }
    }
}

impl Default for ShellPlugin {
    fn default() -> Self {
        let home = home_dir();
        Self::new(
            home.join(DEFAULT_SOURCE_ROOT),
            home.join(DEFAULT_OUTPUT_FILE),
            vec![home.join(".local/bin"), home.join("bin"), home.join(".bin")],
        )
    }
}

impl CodexPlugin for ShellPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        let mut permissions = vec![
            format!("read {}", self.source_root.display()),
            format!("write {}", self.output_path.display()),
        ];
        permissions.extend(
            self.extra_cli_roots
                .iter()
                .map(|root| format!("read {}", root.display())),
        );

        PluginDescriptor {
            id: PLUGIN_ID.to_string(),
            name: "Shell Metadata".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description:
                "Scans shell fragments and user CLI scripts for the desktop shell manager."
                    .to_string(),
            activation: PluginActivation::Eager,
            priority: 700,
            dependencies: vec![PluginDependency {
                id: "builtin/catalog".to_string(),
                optional: false,
            }],
            capabilities: vec![
                "shell-scan".to_string(),
                "cli-page".to_string(),
                "env-page".to_string(),
                "add-fn-catalog".to_string(),
            ],
            permissions,
            kind: PluginKind::Native,
        }
    }

    fn on_enable(&mut self) -> Result<(), codex_plugin_api::PluginError> {
        let scan = scan_shell_sources(&self.source_root, &self.extra_cli_roots);
        let generated_file = managed_generated_file(&scan, &self.source_root, &self.output_path);
        self.scan = Some(scan);
        self.generated_file = Some(generated_file);
        Ok(())
    }

    fn contributions(&self) -> Result<ContributionSet, codex_plugin_api::PluginError> {
        let scan = self
            .scan
            .clone()
            .unwrap_or_else(|| scan_shell_sources(&self.source_root, &self.extra_cli_roots));
        let generated_file = self
            .generated_file
            .clone()
            .unwrap_or_else(|| pending_generated_file(&scan, &self.source_root, &self.output_path));

        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            toolbar_actions: Vec::new(),
            catalog_providers: Vec::new(),
            settings_sections: Vec::new(),
            shell_entries: scan
                .entries
                .iter()
                .map(|entry| entry.contribution.clone())
                .collect(),
            generated_files: vec![generated_file],
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShellScan {
    entries: Vec<ShellEntry>,
    file_count: usize,
}

#[derive(Clone, Debug)]
struct ShellEntry {
    contribution: ShellEntryContribution,
}

pub fn scan_shell_sources(source_root: &Path, extra_cli_roots: &[PathBuf]) -> ShellScan {
    let mut scan = scan_shell_root(source_root);
    for root in extra_cli_roots {
        append_extra_cli_root(&mut scan, root);
    }
    scan
}

pub fn scan_shell_root(root: &Path) -> ShellScan {
    let mut files = Vec::new();
    collect_shell_files(root, root, &mut files);
    files.sort();

    let mut entries = Vec::new();
    for file in files.iter() {
        if let Ok(content) = fs::read_to_string(file) {
            entries.extend(parse_shell_file(root, file, &content));
        }
    }

    ShellScan {
        entries,
        file_count: files.len(),
    }
}

fn append_extra_cli_root(scan: &mut ShellScan, root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    let mut files = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_extra_cli_script(path))
        .collect::<Vec<_>>();
    files.sort();

    for file in files.iter() {
        if let Ok(content) = fs::read_to_string(file) {
            scan.entries.push(shell_entry(
                root,
                file,
                ShellEntryKind::ScriptSnippet,
                script_name(file),
                1,
                &content,
            ));
            scan.file_count += 1;
        }
    }
}

fn collect_shell_files(root: &Path, current_dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if is_ignored_path(&path) {
            continue;
        }
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_dir() {
            collect_shell_files(root, &path, files);
        } else if metadata.is_file() && is_shell_candidate(root, &path) {
            files.push(path);
        }
    }
}

fn is_ignored_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == ".DS_Store" || name.ends_with('~'))
}

fn is_shell_candidate(root: &Path, path: &Path) -> bool {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "sh" | "zsh" | "bash" | "rc"))
    {
        return true;
    }

    path.strip_prefix(root)
        .ok()
        .and_then(|relative| relative.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .is_some_and(|first_component| first_component == "bin")
}

fn is_extra_cli_script(path: &Path) -> bool {
    if !fs::metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return false;
    }

    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("#!"))
}

fn parse_shell_file(root: &Path, file: &Path, content: &str) -> Vec<ShellEntry> {
    let section = source_section(root, file);
    if section == "bin" {
        return vec![shell_entry(
            root,
            file,
            ShellEntryKind::ScriptSnippet,
            script_name(file),
            1,
            content,
        )];
    }

    if should_render_file_as_snippet(&section) {
        let mut entries = extract_metadata_entries(root, file, content);
        entries.push(shell_entry(
            root,
            file,
            ShellEntryKind::ScriptSnippet,
            file_snippet_name(root, file),
            1,
            content,
        ));
        return entries;
    }

    let lines = content.lines().collect::<Vec<_>>();
    let mut entries = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }

        let line_number = index + 1;
        if let Some(name) = parse_alias_name(trimmed) {
            entries.push(shell_entry(
                root,
                file,
                ShellEntryKind::Alias,
                name,
                line_number,
                line,
            ));
            index += 1;
            continue;
        }

        if let Some(name) = parse_export_name(trimmed) {
            entries.push(shell_entry(
                root,
                file,
                ShellEntryKind::Export,
                name,
                line_number,
                line,
            ));
            index += 1;
            continue;
        }

        if let Some(name) = parse_function_name(trimmed) {
            let (body, end_index) = collect_function_body(&lines, index);
            entries.push(shell_entry(
                root,
                file,
                ShellEntryKind::Function,
                name,
                line_number,
                &body,
            ));
            index = end_index + 1;
            continue;
        }

        let (body, end_index) = collect_snippet_body(&lines, index);
        entries.push(shell_entry(
            root,
            file,
            ShellEntryKind::ScriptSnippet,
            snippet_name(file, line_number),
            line_number,
            &body,
        ));
        index = end_index + 1;
    }

    entries
}

fn should_render_file_as_snippet(section: &str) -> bool {
    matches!(
        section,
        "lib"
            | "profile.d"
            | "local.profile.d"
            | "rc.d"
            | "local.rc.d"
            | "zshrc.d"
            | "local.zshrc.d"
    )
}

fn extract_metadata_entries(root: &Path, file: &Path, content: &str) -> Vec<ShellEntry> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut entries = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        let line_number = index + 1;

        if trimmed.is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }

        if let Some(name) = parse_alias_name(trimmed) {
            entries.push(metadata_entry(
                root,
                file,
                ShellEntryKind::Alias,
                name,
                line_number,
                line,
            ));
            index += 1;
            continue;
        }

        if let Some(name) = parse_export_name(trimmed).or_else(|| parse_assignment_name(trimmed)) {
            entries.push(metadata_entry(
                root,
                file,
                ShellEntryKind::Export,
                name,
                line_number,
                line,
            ));
            index += 1;
            continue;
        }

        if let Some(name) = parse_function_name(trimmed) {
            let (body, end_index) = collect_function_body(&lines, index);
            entries.push(metadata_entry(
                root,
                file,
                ShellEntryKind::Function,
                name,
                line_number,
                &body,
            ));
            index = end_index + 1;
            continue;
        }

        index += 1;
    }

    entries
}

fn metadata_entry(
    root: &Path,
    file: &Path,
    kind: ShellEntryKind,
    name: String,
    line_start: usize,
    body: &str,
) -> ShellEntry {
    shell_entry(root, file, kind, name, line_start, body)
}

fn shell_entry(
    root: &Path,
    file: &Path,
    kind: ShellEntryKind,
    name: String,
    line_start: usize,
    body: &str,
) -> ShellEntry {
    let source_path = file.display().to_string();
    let section = source_section(root, file);
    let id = format!(
        "shell.{}.{}.{}",
        kind.label().to_lowercase(),
        sanitize_id(&source_path),
        line_start
    );
    ShellEntry {
        contribution: ShellEntryContribution {
            id,
            kind,
            name,
            section,
            source_path,
            line_start,
            preview: preview_for(kind, body),
            deprecated_source: true,
        },
    }
}

fn parse_alias_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("alias ")?;
    let name = rest.split_once('=')?.0.trim();
    valid_shell_name(name).then(|| name.to_string())
}

fn parse_export_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("export ")?;
    let name = rest
        .split(['=', ' ', '\t'])
        .next()
        .unwrap_or_default()
        .trim();
    valid_shell_name(name).then(|| name.to_string())
}

fn parse_assignment_name(trimmed: &str) -> Option<String> {
    if trimmed.starts_with("local ") || trimmed.starts_with("readonly ") {
        return None;
    }
    let (name, _) = trimmed.split_once('=')?;
    let name = name.trim();
    valid_shell_name(name).then(|| name.to_string())
}

fn parse_function_name(trimmed: &str) -> Option<String> {
    if let Some(rest) = trimmed.strip_prefix("function ") {
        let name = rest
            .split([' ', '\t', '('])
            .next()
            .unwrap_or_default()
            .trim();
        return valid_shell_name(name).then(|| name.to_string());
    }

    let (name, rest) = trimmed.split_once("()")?;
    let name = name.trim();
    (valid_shell_name(name) && rest.trim_start().starts_with('{')).then(|| name.to_string())
}

fn valid_shell_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|char| char == '_' || char.is_ascii_alphanumeric())
}

fn collect_function_body(lines: &[&str], start_index: usize) -> (String, usize) {
    let mut body = Vec::new();
    let mut balance = 0;
    let mut saw_open = false;

    for (index, line) in lines.iter().enumerate().skip(start_index) {
        body.push(*line);
        for char in line.chars() {
            match char {
                '{' => {
                    saw_open = true;
                    balance += 1;
                }
                '}' if balance > 0 => balance -= 1,
                _ => {}
            }
        }
        if saw_open && balance == 0 {
            return (body.join("\n"), index);
        }
    }

    (body.join("\n"), lines.len().saturating_sub(1))
}

fn collect_snippet_body(lines: &[&str], start_index: usize) -> (String, usize) {
    let mut body = Vec::new();
    let mut index = start_index;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || parse_alias_name(trimmed).is_some()
            || parse_export_name(trimmed).is_some()
            || parse_function_name(trimmed).is_some()
        {
            break;
        }
        body.push(lines[index]);
        index += 1;
    }

    (body.join("\n"), index.saturating_sub(1))
}

fn snippet_name(file: &Path, line_start: usize) -> String {
    let file_name = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("shell");
    format!("{file_name}:{line_start}")
}

fn script_name(file: &Path) -> String {
    let name = file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("shell-script")
        .to_string();
    [".bash", ".zsh", ".sh", ".py"]
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(&name)
        .to_string()
}

fn file_snippet_name(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| {
            file.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("shell-fragment")
                .to_string()
        })
}

fn source_section(root: &Path, file: &Path) -> String {
    let Ok(relative) = file.strip_prefix(root) else {
        return section_from_root(root);
    };
    let mut components = relative.components();
    let Some(first) = components.next() else {
        return section_from_root(root);
    };
    let Some(first) = first.as_os_str().to_str() else {
        return section_from_root(root);
    };

    if components.next().is_some() {
        first.to_string()
    } else {
        section_from_root(root)
    }
}

fn section_from_root(root: &Path) -> String {
    let file_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("shell");
    let parent_name = root
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str());

    match (parent_name, file_name) {
        (Some(".local"), "bin") => "local-bin".to_string(),
        (_, "bin") => "home-bin".to_string(),
        (_, ".bin") => "user-bin".to_string(),
        _ => file_name.to_string(),
    }
}

fn preview_for(kind: ShellEntryKind, body: &str) -> String {
    match kind {
        ShellEntryKind::Export => {
            let trimmed = body.trim();
            if let Some((left, _)) = trimmed.split_once('=') {
                format!("{left}=***")
            } else {
                trimmed.to_string()
            }
        }
        _ => body
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or_default()
            .trim()
            .chars()
            .take(140)
            .collect(),
    }
}

fn pending_generated_file(
    scan: &ShellScan,
    source_root: &Path,
    output_path: &Path,
) -> GeneratedFileContribution {
    GeneratedFileContribution {
        id: "shell.add-fn".to_string(),
        path: output_path.display().to_string(),
        source_root: source_root.display().to_string(),
        section_delimiter: SECTION_DELIMITER.to_string(),
        deprecated_source_root: true,
        entry_count: scan.entries.len(),
        backup_path: None,
        status: GeneratedFileStatus::Generated,
        message: "Shell entries are scanned; ~/.add_fn is managed by the visual shell manager."
            .to_string(),
    }
}

fn managed_generated_file(
    scan: &ShellScan,
    source_root: &Path,
    output_path: &Path,
) -> GeneratedFileContribution {
    GeneratedFileContribution {
        id: "shell.add-fn".to_string(),
        path: output_path.display().to_string(),
        source_root: source_root.display().to_string(),
        section_delimiter: SECTION_DELIMITER.to_string(),
        deprecated_source_root: true,
        entry_count: scan.entries.len(),
        backup_path: None,
        status: GeneratedFileStatus::Generated,
        message: format!(
            "Scanned {} entries from {} shell files. ~/.add_fn is only rewritten by the visual manager.",
            scan.entries.len(),
            scan.file_count
        ),
    }
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use codex_plugin_api::{CodexPlugin, GeneratedFileStatus, ShellEntryKind};
    use tempfile::TempDir;

    use super::{ShellPlugin, managed_generated_file, scan_shell_sources};

    #[test]
    fn scanner_extracts_alias_export_function_and_snippet() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        fs::create_dir_all(root.join("rc.d")).expect("create rc dir");
        fs::write(
            root.join("rc.d/test.sh"),
            "alias ll='ls -la'\nexport FOO=bar\nhello_cli() {\n  echo hi\n}\necho snippet\n",
        )
        .expect("write shell file");

        let scan = scan_shell_sources(&root, &[]);
        let kinds = scan
            .entries
            .iter()
            .map(|entry| entry.contribution.kind)
            .collect::<Vec<_>>();

        assert!(kinds.contains(&ShellEntryKind::Alias));
        assert!(kinds.contains(&ShellEntryKind::Export));
        assert!(kinds.contains(&ShellEntryKind::Function));
        assert!(kinds.contains(&ShellEntryKind::ScriptSnippet));
    }

    #[test]
    fn plugin_enable_scans_entries_without_rewriting_add_fn() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        let output = temp.path().join(".add_fn");
        fs::create_dir_all(root.join("profile.d")).expect("create profile dir");
        fs::write(root.join("profile.d/env.sh"), "export TOKEN=secret\n")
            .expect("write shell file");
        fs::write(&output, "manual content").expect("write existing file");

        let mut plugin = ShellPlugin::new(&root, &output, Vec::new());
        plugin.on_enable().expect("enable plugin");
        let content = fs::read_to_string(&output).expect("read generated file");
        let contributions = plugin.contributions().expect("load contributions");

        assert_eq!(content, "manual content");
        assert!(
            contributions
                .shell_entries
                .iter()
                .any(|entry| entry.name == "TOKEN")
        );
        assert_eq!(
            contributions.generated_files[0].status,
            GeneratedFileStatus::Generated
        );
        assert!(
            contributions.generated_files[0]
                .message
                .contains("visual manager")
        );
    }

    #[test]
    fn extra_cli_roots_wrap_shebang_scripts_and_skip_binaries() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        let extra = temp.path().join(".local/bin");
        fs::create_dir_all(&root).expect("create shell root");
        fs::create_dir_all(&extra).expect("create extra root");
        fs::write(
            extra.join("addhost"),
            "#!/usr/bin/env bash\necho addhost \"$@\"\n",
        )
        .expect("write cli script");
        fs::write(
            extra.join("upload-to-minio-addzero.sh"),
            "#!/usr/bin/env zsh\necho upload\n",
        )
        .expect("write zsh cli script");
        fs::write(
            extra.join("nextcloud-dotfiles-link.py"),
            "#!/usr/bin/env python3\nprint('link')\n",
        )
        .expect("write python cli script");
        fs::write(extra.join("binary"), "\0MACHO").expect("write binary marker");

        let scan = scan_shell_sources(&root, &[extra]);
        let names = scan
            .entries
            .iter()
            .map(|entry| entry.contribution.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"addhost"));
        assert!(names.contains(&"upload-to-minio-addzero"));
        assert!(names.contains(&"nextcloud-dotfiles-link"));
        assert!(!names.contains(&"binary"));
    }

    #[test]
    fn scanner_marks_bin_scripts_as_script_snippets() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        let extra = temp.path().join("bin");
        fs::create_dir_all(&root).expect("create shell root");
        fs::create_dir_all(&extra).expect("create extra root");
        fs::write(
            extra.join("danger"),
            "#!/usr/bin/env bash\necho SHOULD_NOT_PRINT\n",
        )
        .expect("write cli script");

        let scan = scan_shell_sources(&root, &[extra]);
        let entry = scan
            .entries
            .iter()
            .find(|entry| entry.contribution.name == "danger")
            .expect("danger entry");

        assert_eq!(entry.contribution.kind, ShellEntryKind::ScriptSnippet);
        assert!(entry.contribution.source_path.ends_with("danger"));
    }

    #[test]
    fn scanner_keeps_zsh_fragments_as_file_snippets() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        fs::create_dir_all(root.join("zshrc.d")).expect("create zsh dir");
        fs::write(root.join("zshrc.d/z.zsh"), "alias -s html='nvim'\n").expect("write zsh file");

        let scan = scan_shell_sources(&root, &[]);
        let entry = scan
            .entries
            .iter()
            .find(|entry| entry.contribution.name == "zshrc.d/z.zsh")
            .expect("zsh file snippet");

        assert_eq!(entry.contribution.kind, ShellEntryKind::ScriptSnippet);
        assert!(entry.contribution.preview.contains("alias -s html"));
    }

    #[test]
    fn scanner_records_aliases_that_collide_with_functions() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        fs::create_dir_all(root.join("rc.d")).expect("create rc dir");
        fs::write(
            root.join("rc.d/duplicates.sh"),
            "alias dkrmi='docker rmi'\ndkrmi() {\n  docker rmi \"$@\"\n}\n",
        )
        .expect("write duplicate shell names");

        let scan = scan_shell_sources(&root, &[]);
        let collisions = scan
            .entries
            .iter()
            .filter(|entry| entry.contribution.name == "dkrmi")
            .count();

        assert_eq!(collisions, 2);
    }

    #[test]
    fn generated_file_metadata_points_to_visual_manager() {
        let temp = TempDir::new().expect("create temp dir");
        let root = temp.path().join("shell");
        fs::create_dir_all(&root).expect("create shell root");
        let scan = scan_shell_sources(&root, &[]);
        let generated = managed_generated_file(&scan, &root, &temp.path().join(".add_fn"));

        assert_eq!(generated.status, GeneratedFileStatus::Generated);
        assert!(generated.message.contains("visual manager"));
    }
}
