use crate::error::Result;
use crate::settings::Settings;

use crate::status::StatusStore;

pub(crate) fn run(settings: &Settings) -> Result<()> {
    StatusStore::new(settings.status_file()).print()
}
