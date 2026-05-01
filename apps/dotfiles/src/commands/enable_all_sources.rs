use crate::error::Result;
use crate::init::enable_all_sources;

pub(crate) fn run() -> Result<()> {
    enable_all_sources()
}
