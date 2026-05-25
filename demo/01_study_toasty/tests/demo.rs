#![allow(unsafe_code)]

use study_toasty::{database_url, run_toasty_crud};
use std::env;

#[test]
fn database_url_reads_env_value() {
    unsafe { env::set_var("TOASTY_DATABASE_URL", "postgres://localhost/test") };
    assert_eq!(database_url().unwrap(), "postgres://localhost/test");
    unsafe { env::remove_var("TOASTY_DATABASE_URL") };
}

#[tokio::test]
async fn run_toasty_crud_integration_if_env_present() {
    let database_url = match env::var("TOASTY_DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return,
    };

    let result = run_toasty_crud(&database_url).await;
    assert!(result.is_ok(), "integration with TOASTY_DATABASE_URL failed: {result:?}");
}
