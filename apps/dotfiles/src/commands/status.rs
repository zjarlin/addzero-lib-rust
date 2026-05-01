use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(settings: &Settings, store: &ConfigStore) -> Result<()> {
    let config = store.load_or_init(settings)?;
    print_current_status(settings, &config);
    Ok(())
}

pub(crate) fn print_current_status(settings: &Settings, config: &crate::config::Config) {
    let platform_config = config.current_platform_config(settings.platform);

    println!("\n当前已配置的软连接:");
    if platform_config.links.is_empty() {
        println!("  暂无软连接配置");
    } else {
        for link in &platform_config.links {
            println!("  {} -> {}", link.source, link.target);
        }
    }

    println!("\n当前已纳入同步的软件包:");
    if platform_config.default_packages.is_empty() {
        println!("  暂无软件包配置");
    } else {
        for package in &platform_config.default_packages {
            println!("  {package}");
        }
    }
}
