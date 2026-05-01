use crate::error::Result;
use crate::init::init_lazyvim;

pub(crate) fn run() -> Result<()> {
    init_lazyvim()
}
