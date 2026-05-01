use crate::error::Result;
use crate::init::init_ohmyzsh;

pub(crate) fn run() -> Result<()> {
    init_ohmyzsh()
}
