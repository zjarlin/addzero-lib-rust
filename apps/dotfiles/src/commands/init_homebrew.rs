use crate::error::Result;
use crate::init::init_homebrew;

pub(crate) fn run() -> Result<()> {
    init_homebrew()
}
