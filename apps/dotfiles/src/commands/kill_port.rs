use crate::error::Result;
use crate::platform::kill_port;

pub(crate) fn run(port: u16) -> Result<()> {
    kill_port(port)?;
    println!("端口 {port} 的占用进程已处理");
    Ok(())
}
