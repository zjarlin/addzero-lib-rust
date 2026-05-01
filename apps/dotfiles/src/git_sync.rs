use std::path::Path;

use crate::config::{Config, SyncType};
use crate::error::{Result, io_error, message};
use crate::platform::{quote_sh, run_shell_checked};
use crate::settings::Settings;

#[derive(Clone, Debug)]
pub struct GitSync<'a> {
    settings: &'a Settings,
    config: &'a Config,
}

impl<'a> GitSync<'a> {
    pub fn new(settings: &'a Settings, config: &'a Config) -> Self {
        Self { settings, config }
    }

    pub fn pull(&self, replace_mismatch: bool) -> Result<()> {
        self.ensure_git_sync()?;
        let cloud_url = self.cloud_url()?;
        let sync_dir = self.sync_dir();

        if !sync_dir.exists() {
            if let Some(parent) = sync_dir.parent() {
                std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
            }
            println!("正在从 {cloud_url} 克隆 dotfiles...");
            run_shell_checked(&format!(
                "git clone {} {}",
                quote_sh(cloud_url),
                quote_sh(&sync_dir.to_string_lossy())
            ))?;
            return Ok(());
        }

        if !sync_dir.join(".git").exists() {
            println!("同步目录已存在但不是 Git 仓库，初始化并关联远端...");
            run_in_dir(sync_dir, "git init")?;
            run_in_dir(
                sync_dir,
                &format!("git remote add origin {}", quote_sh(cloud_url)),
            )?;
        }

        let remote = run_in_dir(sync_dir, "git remote get-url origin");
        match remote {
            Ok(output) => {
                let current_remote = output.stdout.trim();
                if current_remote != cloud_url {
                    if !replace_mismatch {
                        return Err(message(format!(
                            "同步目录 remote 不匹配。当前: {current_remote}; 配置: {cloud_url}。如需替换请加 --replace-mismatch"
                        )));
                    }
                    std::fs::remove_dir_all(sync_dir).map_err(|err| io_error(sync_dir, err))?;
                    return self.pull(false);
                }
            }
            Err(_) => {
                run_in_dir(
                    sync_dir,
                    &format!("git remote add origin {}", quote_sh(cloud_url)),
                )?;
            }
        }

        run_in_dir(sync_dir, "git fetch origin")?;
        let branch = self.detect_remote_branch(sync_dir)?;
        run_in_dir(sync_dir, &format!("git reset --hard origin/{branch}"))?;
        println!("同步目录已更新到 origin/{branch}");
        Ok(())
    }

    pub fn commit_and_push(&self, commit_message: &str) -> Result<()> {
        self.ensure_git_sync()?;
        let sync_dir = self.sync_dir();
        if !sync_dir.exists() {
            return Err(message(format!("同步目录不存在: {}", sync_dir.display())));
        }

        ensure_gitignore(sync_dir)?;
        let status = run_in_dir(sync_dir, "git status --porcelain")?;
        if status.stdout.trim().is_empty() {
            println!("没有需要提交的更改");
            return Ok(());
        }

        println!("检测到以下更改:\n{}", status.stdout.trim());
        run_in_dir(sync_dir, "git add .")?;
        run_in_dir(
            sync_dir,
            &format!("git commit -m {}", quote_sh(commit_message)),
        )?;

        let branch = self.detect_current_or_remote_branch(sync_dir)?;
        let pull = run_in_dir(
            sync_dir,
            &format!("git pull --rebase=false origin {branch}"),
        );
        if pull.is_err() {
            run_in_dir(sync_dir, "git config pull.rebase false")?;
        }
        run_in_dir(sync_dir, &format!("git push origin {branch}"))?;
        println!("成功推送更改到远程仓库");
        Ok(())
    }

    fn ensure_git_sync(&self) -> Result<()> {
        if self.config.sync_type != SyncType::Git {
            return Err(message("当前配置的同步类型不是 GIT"));
        }
        Ok(())
    }

    fn cloud_url(&self) -> Result<&str> {
        self.config
            .cloud_url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
            .ok_or_else(|| message("请先设置云端仓库地址: dotfiles config set-cloud-url <url>"))
    }

    fn sync_dir(&self) -> &Path {
        if self.config.sync_dir.is_empty() {
            &self.settings.sync_dir
        } else {
            Path::new(&self.config.sync_dir)
        }
    }

    fn detect_remote_branch(&self, dir: &Path) -> Result<String> {
        let output = run_in_dir(dir, "git branch -r")?.stdout;
        if output.contains("origin/master") {
            Ok("master".to_string())
        } else if output.contains("origin/main") {
            Ok("main".to_string())
        } else {
            Ok("master".to_string())
        }
    }

    fn detect_current_or_remote_branch(&self, dir: &Path) -> Result<String> {
        let output = run_in_dir(dir, "git branch --show-current")?;
        let current = output.stdout.trim();
        if current.is_empty() {
            self.detect_remote_branch(dir)
        } else {
            Ok(current.to_string())
        }
    }
}

fn run_in_dir(dir: &Path, command: &str) -> Result<crate::platform::CommandOutput> {
    let command = format!("cd {} && {command}", quote_sh(&dir.to_string_lossy()));
    run_shell_checked(&command)
}

fn ensure_gitignore(sync_dir: &Path) -> Result<()> {
    let path = sync_dir.join(".gitignore");
    if path.exists() {
        return Ok(());
    }

    let content = r#".gradle
build/
!gradle/wrapper/gradle-wrapper.jar
!**/src/main/**/build/
!**/src/test/**/build/

### IntelliJ IDEA ###
.idea
*.iws
*.iml
*.ipr
out/
!**/src/main/**/out/
!**/src/test/**/out/

### Kotlin ###
.kotlin

### Eclipse ###
.apt_generated
.classpath
.factorypath
.project
.settings
.springBeans
.sts4-cache
bin/
!**/src/main/**/bin/
!**/src/test/**/bin/

### NetBeans ###
/nbproject/private/
/nbbuild/
/dist/
/nbdist/
/.nb-gradle/

### VS Code ###
.vscode/

### Mac OS ###
.DS_Store
"#;

    std::fs::write(&path, content).map_err(|err| io_error(&path, err))?;
    Ok(())
}
