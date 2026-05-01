use crate::error::Result;
use crate::init::init_docker;

pub(crate) fn run() -> Result<()> {
    init_docker()
}
