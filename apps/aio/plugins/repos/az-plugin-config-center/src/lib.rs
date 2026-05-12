#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

//! Configuration center plugin runtime.
//!
//! The host loads this plugin from an `.azplugin` bundle. The current runtime
//! exports lifecycle hooks while the page metadata and table surfaces live in
//! `plugin.toml`; future host operations can bind these hooks to approved config
//! center reads, shell sync previews, and audited writes.

/// Returns the managed shell config root used by this plugin profile.
pub fn managed_shell_root() -> &'static str {
    "~/.config/shell"
}

/// Returns the source workbook path used to seed metadata for the current profile.
pub fn metadata_workbook_path() -> &'static str {
    "Archive/biz_dotfiles.xlsx"
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_load() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_enable() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_disable() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_unload() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::{managed_shell_root, metadata_workbook_path};

    #[test]
    fn declares_shell_root_and_workbook_source() {
        assert_eq!(managed_shell_root(), "~/.config/shell");
        assert!(metadata_workbook_path().ends_with("biz_dotfiles.xlsx"));
    }
}
