use crate::error::Result;
use crate::platform::show_port;

pub(crate) fn run(port: u16) -> Result<()> {
    println!("{}", show_port(port)?);
    Ok(())
}
