#![cfg(target_os = "macos")]

use anyhow::{Context, Result};
use az_derive_aliases::{apply, serialize_debug};
use az_str::api::escape_xml;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const HOST_WORKFLOW_NAME: &str = "AIO Drive 托管.workflow";
const UNHOST_WORKFLOW_NAME: &str = "AIO Drive 取消托管.workflow";
const HOST_MENU_LABEL: &str = "AIO Drive 托管";
const UNHOST_MENU_LABEL: &str = "AIO Drive 取消托管";

#[apply(serialize_debug)]
pub struct MacosActionsInstallResult {
    workflows: Vec<PathBuf>,
    scripts: Vec<PathBuf>,
    config_paths: Vec<PathBuf>,
    enabled_context_menu: bool,
    refreshed_services_cache: bool,
    note: String,
}

pub fn install() -> Result<MacosActionsInstallResult> {
    let binary = env::current_exe().context("failed to resolve current az-drive-app binary")?;
    let home = home_dir().context("HOME is required to install Finder Quick Actions")?;
    let services_dir = home.join("Library").join("Services");
    let support_dir = home
        .join("Library")
        .join("Application Support")
        .join("az-drive")
        .join("finder-actions");
    fs::create_dir_all(&services_dir).context("failed to create ~/Library/Services")?;
    fs::create_dir_all(&support_dir).context("failed to create Finder action script dir")?;

    let host_script = support_dir.join("host.sh");
    let unhost_script = support_dir.join("unhost.sh");
    write_script(&host_script, &binary, "host", "托管")?;
    write_script(&unhost_script, &binary, "unhost", "取消托管")?;

    let host_workflow = services_dir.join(HOST_WORKFLOW_NAME);
    let unhost_workflow = services_dir.join(UNHOST_WORKFLOW_NAME);
    write_workflow(
        &host_workflow,
        HOST_MENU_LABEL,
        &format!("exec {} \"$@\"", zsh_single_quote(&host_script)),
    )?;
    write_workflow(
        &unhost_workflow,
        UNHOST_MENU_LABEL,
        &format!("exec {} \"$@\"", zsh_single_quote(&unhost_script)),
    )?;

    let enabled_host = enable_context_menu(HOST_MENU_LABEL);
    let enabled_unhost = enable_context_menu(UNHOST_MENU_LABEL);
    let refreshed = refresh_services_cache();
    Ok(MacosActionsInstallResult {
        workflows: vec![host_workflow, unhost_workflow],
        scripts: vec![host_script, unhost_script],
        config_paths: crate::drive_config_paths(),
        enabled_context_menu: enabled_host && enabled_unhost,
        refreshed_services_cache: refreshed,
        note: "Finder 右键入口已请求加入右键菜单和“快速操作”；如果未立即出现，请重新打开 Finder 右键菜单或重启 Finder。"
            .to_owned(),
    })
}

fn write_script(path: &Path, binary: &Path, command: &str, label: &str) -> Result<()> {
    let log = "${HOME}/Library/Logs/az-drive-finder-actions.log";
    let binary = zsh_single_quote(binary);
    let content = format!(
        r#"#!/bin/zsh
set -u

BIN={binary}
LOG="{log}"
DRIVE_TOML="${{HOME}}/.config/aio/drive.toml"
AUTH_JSON="${{HOME}}/.config/aio/auth.json"

notify() {{
  /usr/bin/osascript -e "display notification \"$1\" with title \"AIO Drive\"" >/dev/null 2>&1 || true
}}

mkdir -p "${{HOME}}/Library/Logs"

{{
  echo "[$(/bin/date -u '+%Y-%m-%dT%H:%M:%SZ')] {label}: $# item(s)"
}} >> "$LOG"

if [[ $# -eq 0 ]]; then
  notify "Finder 没有传入选中文件"
  exit 0
fi

if [[ ! -f "$DRIVE_TOML" || ! -f "$AUTH_JSON" ]]; then
  echo "missing drive config; checked: $DRIVE_TOML $AUTH_JSON" >> "$LOG"
  notify "未配置 AIO Drive，无法执行{label}"
  exit 2
fi

failed=0
for selected_path in "$@"; do
  echo "{label}: $selected_path" >> "$LOG"
  "$BIN" {command} "$selected_path" >> "$LOG" 2>&1 || failed=1
done

if [[ "$failed" -eq 0 ]]; then
  notify "{label}完成"
else
  notify "{label}失败，查看 ~/Library/Logs/az-drive-finder-actions.log"
fi
exit "$failed"
"#
    );
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("failed to chmod {}", path.display()))?;
    Ok(())
}

fn write_workflow(path: &Path, menu_label: &str, command_string: &str) -> Result<()> {
    let contents_dir = path.join("Contents");
    fs::create_dir_all(&contents_dir)
        .with_context(|| format!("failed to create {}", contents_dir.display()))?;
    let document = workflow_document(command_string);
    let document_path = contents_dir.join("document.wflow");
    fs::write(&document_path, document)
        .with_context(|| format!("failed to write {}", document_path.display()))?;
    let info_path = contents_dir.join("Info.plist");
    fs::write(&info_path, info_plist(menu_label))
        .with_context(|| format!("failed to write {}", info_path.display()))?;
    Ok(())
}

fn info_plist(menu_label: &str) -> String {
    let label = escape_xml(menu_label);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>NSServices</key>
  <array>
    <dict>
      <key>NSBackgroundColorName</key><string>background</string>
      <key>NSIconName</key><string>NSActionTemplate</string>
      <key>NSMenuItem</key><dict><key>default</key><string>{label}</string></dict>
      <key>NSMessage</key><string>runWorkflowAsService</string>
      <key>NSRequiredContext</key><dict><key>NSApplicationIdentifier</key><string>com.apple.finder</string></dict>
      <key>NSSendFileTypes</key><array><string>public.item</string></array>
    </dict>
  </array>
</dict>
</plist>
"#
    )
}

fn workflow_document(command_string: &str) -> String {
    let command = escape_xml(command_string);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>AMApplicationBuild</key><string>528</string>
  <key>AMApplicationVersion</key><string>2.10</string>
  <key>AMDocumentVersion</key><string>2</string>
  <key>actions</key>
  <array>
    <dict>
      <key>action</key>
      <dict>
        <key>AMAccepts</key>
        <dict>
          <key>Container</key><string>List</string>
          <key>Optional</key><true/>
          <key>Types</key><array><string>com.apple.cocoa.string</string></array>
        </dict>
        <key>AMActionVersion</key><string>2.0.3</string>
        <key>AMApplication</key><array><string>Automator</string></array>
        <key>AMParameterProperties</key>
        <dict>
          <key>COMMAND_STRING</key><dict/>
          <key>CheckedForUserDefaultShell</key><dict/>
          <key>inputMethod</key><dict/>
          <key>shell</key><dict/>
          <key>source</key><dict/>
        </dict>
        <key>AMProvides</key>
        <dict>
          <key>Container</key><string>List</string>
          <key>Types</key><array><string>com.apple.cocoa.string</string></array>
        </dict>
        <key>ActionBundlePath</key><string>/System/Library/Automator/Run Shell Script.action</string>
        <key>ActionName</key><string>Run Shell Script</string>
        <key>ActionParameters</key>
        <dict>
          <key>COMMAND_STRING</key><string>{command}</string>
          <key>CheckedForUserDefaultShell</key><true/>
          <key>inputMethod</key><integer>1</integer>
          <key>shell</key><string>/bin/zsh</string>
          <key>source</key><string></string>
        </dict>
        <key>BundleIdentifier</key><string>com.apple.RunShellScript</string>
        <key>CFBundleVersion</key><string>2.0.3</string>
        <key>CanShowSelectedItemsWhenRun</key><false/>
        <key>CanShowWhenRun</key><true/>
        <key>Category</key><array><string>AMCategoryUtilities</string></array>
        <key>Class Name</key><string>RunShellScriptAction</string>
        <key>InputUUID</key><string>777F4E09-36C9-42B8-A92A-20A78C2D8F9B</string>
        <key>Keywords</key><array><string>Shell</string><string>Script</string><string>Run</string><string>Unix</string></array>
        <key>OutputUUID</key><string>1FBB9155-6E74-4C3A-9C77-AEF9848A90D8</string>
        <key>UUID</key><string>82693830-D471-4D28-AD4D-9D99141D1248</string>
        <key>UnlocalizedApplications</key><array><string>Automator</string></array>
        <key>arguments</key>
        <dict>
          <key>0</key><dict><key>default value</key><integer>0</integer><key>name</key><string>inputMethod</string><key>required</key><string>0</string><key>type</key><string>0</string><key>uuid</key><string>0</string></dict>
          <key>1</key><dict><key>default value</key><false/><key>name</key><string>CheckedForUserDefaultShell</string><key>required</key><string>0</string><key>type</key><string>0</string><key>uuid</key><string>1</string></dict>
          <key>2</key><dict><key>default value</key><string></string><key>name</key><string>source</string><key>required</key><string>0</string><key>type</key><string>0</string><key>uuid</key><string>2</string></dict>
          <key>3</key><dict><key>default value</key><string></string><key>name</key><string>COMMAND_STRING</string><key>required</key><string>0</string><key>type</key><string>0</string><key>uuid</key><string>3</string></dict>
          <key>4</key><dict><key>default value</key><string>/bin/sh</string><key>name</key><string>shell</string><key>required</key><string>0</string><key>type</key><string>0</string><key>uuid</key><string>4</string></dict>
        </dict>
        <key>isViewVisible</key><true/>
        <key>location</key><string>309.000000:305.000000</string>
        <key>nibPath</key><string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib</string>
      </dict>
      <key>isViewVisible</key><true/>
    </dict>
  </array>
  <key>connectors</key><dict/>
  <key>state</key><dict><key>AMLogTabViewSelectedIndex</key><integer>0</integer></dict>
  <key>workflowMetaData</key>
  <dict>
    <key>applicationBundleID</key><string>com.apple.finder</string>
    <key>applicationBundleIDsByPath</key><dict><key>/System/Library/CoreServices/Finder.app</key><string>com.apple.finder</string></dict>
    <key>applicationPath</key><string>/System/Library/CoreServices/Finder.app</string>
    <key>applicationPaths</key><array><string>/System/Library/CoreServices/Finder.app</string></array>
    <key>inputTypeIdentifier</key><string>com.apple.Automator.fileSystemObject</string>
    <key>outputTypeIdentifier</key><string>com.apple.Automator.nothing</string>
    <key>presentationMode</key><integer>15</integer>
    <key>processesInput</key><true/>
    <key>serviceApplicationBundleID</key><string>com.apple.finder</string>
    <key>serviceApplicationPath</key><string>/System/Library/CoreServices/Finder.app</string>
    <key>serviceInputTypeIdentifier</key><string>com.apple.Automator.fileSystemObject</string>
    <key>serviceOutputTypeIdentifier</key><string>com.apple.Automator.nothing</string>
    <key>serviceProcessesInput</key><true/>
    <key>systemImageName</key><string>externaldrive</string>
    <key>useAutomaticInputType</key><false/>
    <key>workflowTypeIdentifier</key><string>com.apple.Automator.servicesMenu</string>
  </dict>
</dict>
</plist>
"#
    )
}

fn refresh_services_cache() -> bool {
    Command::new("/System/Library/CoreServices/pbs")
        .arg("-flush")
        .arg("en")
        .arg("zh")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn enable_context_menu(menu_label: &str) -> bool {
    let service_key = format!("(null) - {menu_label} - runWorkflowAsService");
    Command::new("/usr/bin/defaults")
        .arg("write")
        .arg("pbs")
        .arg("NSServicesStatus")
        .arg("-dict-add")
        .arg(defaults_string_key(&service_key))
        .arg("{ presentation_modes = { ContextMenu = 1; FinderPreview = 1; ServicesMenu = 1; TouchBar = 1; }; }")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn defaults_string_key(value: &str) -> String {
    format!("'{}'", value.replace('\'', "\\'"))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn zsh_single_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
