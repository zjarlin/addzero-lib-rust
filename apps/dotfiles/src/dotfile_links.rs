use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{Config, LinkLine};
use crate::error::{Result, io_error, message};
use crate::settings::{Settings, expand_home};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkApplyMode {
    SkipExisting,
    BackupAndReplace,
}

impl LinkApplyMode {
    pub fn from_force(force: bool) -> Self {
        if force {
            Self::BackupAndReplace
        } else {
            Self::SkipExisting
        }
    }
}

pub fn add_dotfiles(
    settings: &Settings,
    config: &mut Config,
    paths: &[String],
    abs: bool,
) -> Result<bool> {
    let mut changed = false;
    for input in paths {
        let source = resolve_source_path(settings, input, abs)?;
        let target = resolve_target_path(settings, &source);
        move_into_dotfiles_and_link(&source, &target)?;

        let line = LinkLine {
            source: source.to_string_lossy().into_owned(),
            target: target.to_string_lossy().into_owned(),
        };

        if config
            .current_platform_config_mut(settings.platform)
            .links
            .insert(line)
        {
            changed = true;
            println!("文件添加完成: {}", source.display());
        } else {
            println!("文件 {} 已经配置过，跳过添加", source.display());
        }
    }

    Ok(changed)
}

pub fn remove_dotfiles(
    settings: &Settings,
    config: &mut Config,
    paths: &[String],
    abs: bool,
) -> Result<bool> {
    let mut changed = false;
    for input in paths {
        let source = resolve_source_path_for_removal(settings, input, abs);
        let source_text = source.to_string_lossy().into_owned();
        let target = config
            .current_platform_config(settings.platform)
            .links
            .iter()
            .find(|link| link.source == source_text)
            .map(|link| PathBuf::from(&link.target));

        undo_link(&source, target.as_deref())?;

        let links = &mut config.current_platform_config_mut(settings.platform).links;
        let before = links.len();
        links.retain(|link| link.source != source_text);
        if links.len() != before {
            changed = true;
            println!("已从配置中移除链接: {source_text}");
        } else {
            println!("未在配置中找到链接: {source_text}");
        }
    }
    Ok(changed)
}

pub fn apply_configured_links(
    settings: &Settings,
    config: &Config,
    mode: LinkApplyMode,
) -> Result<()> {
    for link in &config.current_platform_config(settings.platform).links {
        let source = PathBuf::from(&link.source);
        let target = PathBuf::from(&link.target);
        if !target.exists() {
            println!("跳过缺失目标: {}", target.display());
            continue;
        }
        create_link_from_target(&source, &target, mode)?;
    }
    Ok(())
}

fn resolve_source_path(settings: &Settings, input: &str, abs: bool) -> Result<PathBuf> {
    let path = resolve_source_path_for_removal(settings, input, abs);
    if !path.exists() {
        return Err(message(format!("dotfiles 文件不存在: {}", path.display())));
    }
    Ok(path)
}

fn resolve_source_path_for_removal(settings: &Settings, input: &str, abs: bool) -> PathBuf {
    let already_absolute = is_absolute_like(input);
    let path = if abs || already_absolute {
        expand_home(input, &settings.home_dir)
    } else {
        settings.home_dir.join(input)
    };
    normalize_path(path)
}

fn resolve_target_path(settings: &Settings, source: &Path) -> PathBuf {
    if let Ok(relative) = source.strip_prefix(&settings.home_dir) {
        settings.dotfiles_dir.join(relative)
    } else {
        settings.dotfiles_dir.join(
            source
                .file_name()
                .map(std::ffi::OsStr::to_os_string)
                .unwrap_or_default(),
        )
    }
}

fn move_into_dotfiles_and_link(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        if is_symlink_to(source, target)? {
            return Ok(());
        }
        return Err(message(format!(
            "目标已存在，避免覆盖: {}",
            target.display()
        )));
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    }

    std::fs::rename(source, target).map_err(|err| io_error(source, err))?;
    create_symlink(target, source)?;
    println!("已创建软链接: {} -> {}", source.display(), target.display());
    Ok(())
}

fn create_link_from_target(source: &Path, target: &Path, mode: LinkApplyMode) -> Result<()> {
    if source.exists() || source.symlink_metadata().is_ok() {
        if is_symlink_to(source, target)? {
            return Ok(());
        }
        match mode {
            LinkApplyMode::SkipExisting => {
                println!("跳过已有路径: {}", source.display());
                return Ok(());
            }
            LinkApplyMode::BackupAndReplace => {
                let backup = backup_path(source);
                std::fs::rename(source, &backup).map_err(|err| io_error(source, err))?;
                println!("已备份 {} 到 {}", source.display(), backup.display());
            }
        }
    }

    if let Some(parent) = source.parent() {
        std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
    }
    create_symlink(target, source)?;
    println!("已创建软链接: {} -> {}", source.display(), target.display());
    Ok(())
}

fn undo_link(source: &Path, target: Option<&Path>) -> Result<()> {
    match source.symlink_metadata() {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            std::fs::remove_file(source).map_err(|err| io_error(source, err))?;
            println!("已删除软链接: {}", source.display());
        }
        Ok(_) => {
            println!("源路径不是软链接，保留文件: {}", source.display());
            return Ok(());
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(io_error(source, err)),
    }

    if let Some(target) = target {
        if target.exists() {
            if source.exists() || source.symlink_metadata().is_ok() {
                return Err(message(format!(
                    "无法恢复 {}，源路径已存在",
                    source.display()
                )));
            }
            if let Some(parent) = source.parent() {
                std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
            }
            std::fs::rename(target, source).map_err(|err| io_error(target, err))?;
            println!("已恢复文件: {} <- {}", source.display(), target.display());
        }
    }

    Ok(())
}

fn is_symlink_to(source: &Path, target: &Path) -> Result<bool> {
    let metadata = match source.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(io_error(source, err)),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(false);
    }

    let read_target = std::fs::read_link(source).map_err(|err| io_error(source, err))?;
    Ok(read_target == target)
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link).map_err(|err| io_error(link, err))
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> Result<()> {
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link).map_err(|err| io_error(link, err))
    } else {
        std::os::windows::fs::symlink_file(target, link).map_err(|err| io_error(link, err))
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn backup_path(source: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let file_name = source
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "backup".into());
    source.with_file_name(format!("{file_name}.bak.{timestamp}"))
}

fn is_absolute_like(input: &str) -> bool {
    Path::new(input).is_absolute()
        || input.starts_with('~')
        || input.starts_with("${HOME}")
        || input.starts_with("$HOME")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn resolve_target_preserves_home_relative_path() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let settings = Settings::for_home(temp.path());
        let source = temp.path().join(".zshrc");

        let target = resolve_target_path(&settings, &source);

        assert_eq!(target, settings.dotfiles_dir.join(".zshrc"));
    }

    #[test]
    fn add_dotfile_moves_file_and_records_link() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let settings = Settings::for_home(temp.path());
        let source = temp.path().join(".zshrc");
        std::fs::write(&source, "source ~/.config/shell/profile").expect("write source");
        let mut config = Config::default_for_settings(&settings);

        let changed =
            add_dotfiles(&settings, &mut config, &[".zshrc".to_string()], false).expect("add");

        assert!(changed);
        assert!(
            source
                .symlink_metadata()
                .expect("metadata")
                .file_type()
                .is_symlink()
        );
        assert!(settings.dotfiles_dir.join(".zshrc").exists());
        assert_eq!(
            config
                .current_platform_config(settings.platform)
                .links
                .iter()
                .next()
                .expect("link")
                .target,
            settings.dotfiles_dir.join(".zshrc").to_string_lossy()
        );
    }

    #[test]
    fn remove_dotfile_restores_target_to_source() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let settings = Settings::for_home(temp.path());
        let source = temp.path().join(".zshrc");
        std::fs::write(&source, "managed").expect("write source");
        let mut config = Config::default_for_settings(&settings);
        add_dotfiles(&settings, &mut config, &[".zshrc".to_string()], false).expect("add");

        let changed = remove_dotfiles(&settings, &mut config, &[".zshrc".to_string()], false)
            .expect("remove");

        assert!(changed);
        assert!(
            !source
                .symlink_metadata()
                .expect("metadata")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            std::fs::read_to_string(&source).expect("read source"),
            "managed"
        );
        assert!(
            config
                .current_platform_config(settings.platform)
                .links
                .is_empty()
        );
    }
}
