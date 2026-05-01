mod install;
mod search;
mod version;

use crate::cli::PackageSubcommand;
use crate::config::ConfigStore;
use crate::error::Result;
use crate::settings::Settings;

pub(crate) fn run(
    settings: &Settings,
    store: &ConfigStore,
    command: PackageSubcommand,
) -> Result<()> {
    match command {
        PackageSubcommand::Install { packages } => install::run(settings, store, packages),
        PackageSubcommand::Search { keyword } => search::run(settings, store, keyword),
        PackageSubcommand::Version { package } => version::run(settings, store, package),
    }
}
