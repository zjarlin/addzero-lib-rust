//! GitDB - A Git-backed Document Database
//!
//! This is the main entry point for the GitDB command-line interface.

use std::path::PathBuf;
use std::process::ExitCode;

use gitdb::db::{Database, DatabaseConfig, Repl};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    // Parse simple command line args.
    let mut path = PathBuf::from(".gitdb");
    let mut verbose = false;
    let mut execute: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--database" => {
                i += 1;
                if i < args.len() {
                    path = PathBuf::from(&args[i]);
                }
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "-e" | "--execute" => {
                i += 1;
                if i < args.len() {
                    execute = Some(args[i].clone());
                }
            }
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "--version" => {
                println!("GitDB v0.1.0");
                return ExitCode::SUCCESS;
            }
            arg => {
                // Treat as database path if no flag.
                if !arg.starts_with('-') {
                    path = PathBuf::from(arg);
                } else {
                    eprintln!("Unknown option: {}", arg);
                    return ExitCode::FAILURE;
                }
            }
        }
        i += 1;
    }

    // Open database.
    let config = DatabaseConfig::new(&path)
        .create_if_missing(true)
        .verbose(verbose);

    let db = match Database::open_with_config(config) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Execute single command or run REPL.
    if let Some(sql) = execute {
        match execute_command(db, &sql) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        }
    } else {
        match run_repl(db) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        }
    }
}

fn print_help() {
    println!("GitDB - A Git-backed Document Database");
    println!();
    println!("Usage: gitdb [OPTIONS] [DATABASE]");
    println!();
    println!("Options:");
    println!("  -d, --database PATH    Path to database directory (default: .gitdb)");
    println!("  -e, --execute SQL      Execute SQL and exit");
    println!("  -v, --verbose          Enable verbose output");
    println!("  -h, --help             Show this help message");
    println!("  --version              Show version");
    println!();
    println!("Examples:");
    println!("  gitdb                           Start REPL with default database");
    println!("  gitdb mydb                      Start REPL with 'mydb' database");
    println!("  gitdb -e 'SELECT * FROM users'  Execute query and exit");
}

fn execute_command(mut db: Database, sql: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = db.execute(sql)?;
    print_result(&result);
    Ok(())
}

fn run_repl(db: Database) -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new(db);
    repl.run()?;
    Ok(())
}

fn print_result(result: &gitdb::executor::QueryResult) {
    use gitdb::executor::QueryResult;

    match result {
        QueryResult::Success { message } => {
            println!("{}", message);
        }
        QueryResult::Modified { rows_affected } => {
            println!("{} row(s) modified", rows_affected);
        }
        QueryResult::Select(rs) => {
            if rs.is_empty() {
                println!("(0 rows)");
                return;
            }

            // Simple output for CLI.
            let columns: Vec<&String> = rs.rows[0].keys().collect();
            println!(
                "{}",
                columns
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join("\t")
            );

            for row in &rs.rows {
                let values: Vec<String> = columns
                    .iter()
                    .map(|col| row.get(*col).map(format_value).unwrap_or_default())
                    .collect();
                println!("{}", values.join("\t"));
            }
            println!("({} rows)", rs.len());
        }
        QueryResult::Transaction { message } => {
            println!("{}", message);
        }
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => value.to_string(),
    }
}
