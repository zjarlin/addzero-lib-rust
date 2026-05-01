use crate::error::Result;
use crate::platform::run_checked;

pub(crate) fn run(username: String, email: String) -> Result<()> {
    run_checked("git", ["config", "--global", "user.name", &username])?;
    run_checked("git", ["config", "--global", "user.email", &email])?;
    println!("Git 用户名已设置: {username}");
    println!("Git 邮箱已设置: {email}");
    Ok(())
}
