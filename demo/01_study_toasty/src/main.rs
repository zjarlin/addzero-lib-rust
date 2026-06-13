#![forbid(unsafe_code)]

use toasty_driver_sqlite::Sqlite;

#[derive(Debug, toasty::Model)]
struct User {
    #[key]
    #[auto]
    id: u64,

    name: String,

    #[unique]
    email: String,
}

#[tokio::main]
async fn main() -> toasty::Result<()> {
    let driver = Sqlite::in_memory();
    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .build(driver)
        .await?;

    db.push_schema().await?;

    let user = toasty::create!(User {
        name: "Alice",
        email: "alice@example.com",
    })
    .exec(&mut db)
    .await?;

    println!("Created: {:?}", user.name);

    // Fetch the user back by primary key
    let found = User::get_by_id(&mut db, &user.id).await?;
    println!("Found: {:?}", found.email);

    Ok(())
}
