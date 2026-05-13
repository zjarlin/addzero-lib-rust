#![forbid(unsafe_code)]

rust_i18n::i18n!("locales", fallback = "en");

mod app;
mod embedded_backend;

pub use embedded_backend::{DesktopRuntime, DesktopRuntimeOptions};

pub(crate) const DEFAULT_LOCALE: &str = "zh-CN";

pub fn run() -> anyhow::Result<()> {
    app::run()
}

#[cfg(test)]
mod tests {
    use rust_i18n::t;

    #[test]
    fn desktop_translations_default_to_chinese_with_english_fallback() {
        rust_i18n::set_locale("zh-CN");
        assert_eq!(t!("toolbar.output_path"), "输出路径");

        rust_i18n::set_locale("en");
        assert_eq!(t!("toolbar.output_path"), "Output Path");

        rust_i18n::set_locale("fr");
        assert_eq!(t!("toolbar.output_path"), "Output Path");
    }
}
