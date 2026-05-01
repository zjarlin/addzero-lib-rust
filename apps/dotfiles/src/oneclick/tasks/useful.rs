use std::path::PathBuf;

use crate::settings::Settings;

pub(crate) fn useful_soft_dir(settings: &Settings) -> PathBuf {
    settings.work_dir.join("useful")
}
