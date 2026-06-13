//! Interactive REPL (Read-Eval-Print Loop) for GitDB.

use std::io::{self, BufRead, Write};

use super::api::Database;
use crate::executor::QueryResult;
use az_derive_aliases::{apply, impl_default, plain_clone_debug};

/// REPL configuration.
#[apply(plain_clone_debug)]
pub struct ReplConfig {
    /// Prompt string.
    pub prompt: String,
    /// Show timing information.
    pub timing: bool,
    /// Enable colors in output.
    pub colors: bool,
    /// Max rows to display.
    pub max_rows: usize,
}

impl_default!(ReplConfig => ReplConfig {
    prompt: "gitdb> ".into(),
    timing: true,
    colors: true,
    max_rows: 100,
});

/// The interactive REPL.
pub struct Repl {
    db: Database,
    config: ReplConfig,
    history: Vec<String>,
}

impl Repl {
    /// Create a new REPL with the given database.
    pub fn new(db: Database) -> Self {
        Self {
            db,
            config: ReplConfig::default(),
            history: Vec::new(),
        }
    }

    /// Create a REPL with custom configuration.
    pub fn with_config(db: Database, config: ReplConfig) -> Self {
        Self {
            db,
            config,
            history: Vec::new(),
        }
    }

    /// Run the REPL interactively.
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.print_banner();

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut input = String::new();
        let mut multiline = false;

        loop {
            // Print prompt.
            let prompt = if multiline {
                "     -> "
            } else {
                &self.config.prompt
            };
            print!("{}", prompt);
            stdout.flush()?;

            // Read line.
            let mut line = String::new();
            if stdin.lock().read_line(&mut line)? == 0 {
                // EOF.
                println!("\nGoodbye!");
                break;
            }

            let line = line.trim_end();

            // Handle empty lines.
            if line.is_empty() && !multiline {
                continue;
            }

            // Accumulate multi-line input.
            if multiline {
                input.push(' ');
            }
            input.push_str(line);

            // Check for continuation (line ends without semicolon for SQL).
            if !input.ends_with(';') && !self.is_command(&input) {
                multiline = true;
                continue;
            }
            multiline = false;

            // Process input.
            let cmd = input.trim().to_string();
            input.clear();

            if cmd.is_empty() {
                continue;
            }

            // Add to history.
            self.history.push(cmd.clone());

            // Handle special commands.
            if self.is_command(&cmd) {
                match self.handle_command(&cmd) {
                    Ok(should_exit) if should_exit => break,
                    Err(e) => eprintln!("Error: {}", e),
                    _ => {}
                }
                continue;
            }

            // Execute SQL.
            let start = std::time::Instant::now();
            match self.db.execute(&cmd) {
                Ok(result) => {
                    self.print_result(&result);
                    if self.config.timing {
                        println!("Time: {:.3}ms", start.elapsed().as_secs_f64() * 1000.0);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Ok(())
    }

    fn print_banner(&self) {
        println!("╔═══════════════════════════════════════════════════╗");
        println!("║                     GitDB v0.1.0                  ║");
        println!("║         A Git-backed Document Database            ║");
        println!("╠═══════════════════════════════════════════════════╣");
        println!("║  Type .help for commands, or enter SQL statements ║");
        println!("╚═══════════════════════════════════════════════════╝");
        println!();
    }

    fn is_command(&self, input: &str) -> bool {
        input.starts_with('.') || input.starts_with('\\')
    }

    fn handle_command(&mut self, cmd: &str) -> anyhow::Result<bool> {
        let cmd = cmd.trim_start_matches(&['.', '\\'][..]);
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts.first().map(|s| s.to_lowercase());

        match command.as_deref() {
            Some("help") | Some("h") | Some("?") => {
                self.print_help();
            }
            Some("quit") | Some("exit") | Some("q") => {
                return Ok(true);
            }
            Some("tables") | Some("dt") => {
                self.list_tables()?;
            }
            Some("schema") | Some("describe") | Some("d") => {
                if let Some(table) = parts.get(1) {
                    self.describe_table(table)?;
                } else {
                    eprintln!("Usage: .schema <table_name>");
                }
            }
            Some("stats") => {
                self.print_stats();
            }
            Some("history") => {
                self.print_history();
            }
            Some("explain") => {
                let sql = parts[1..].join(" ");
                if sql.is_empty() {
                    eprintln!("Usage: .explain <sql>");
                } else {
                    match self.db.explain(&sql) {
                        Ok(plan) => println!("{}", plan),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
            Some("clear") => {
                // Clear screen (ANSI escape).
                print!("\x1B[2J\x1B[H");
            }
            Some("timing") => {
                self.config.timing = !self.config.timing;
                println!("Timing: {}", if self.config.timing { "on" } else { "off" });
            }
            Some(cmd) => {
                eprintln!("Unknown command: .{}", cmd);
                eprintln!("Type .help for available commands");
            }
            None => {}
        }

        Ok(false)
    }

    fn print_help(&self) {
        println!("Commands:");
        println!("  .help, .h, .?           Show this help message");
        println!("  .quit, .exit, .q        Exit the REPL");
        println!("  .tables, .dt            List all tables");
        println!("  .schema <table>         Show table schema");
        println!("  .stats                  Show database statistics");
        println!("  .history                Show command history");
        println!("  .explain <sql>          Show query execution plan");
        println!("  .timing                 Toggle timing display");
        println!("  .clear                  Clear the screen");
        println!();
        println!("SQL Statements:");
        println!("  CREATE TABLE name (columns...)");
        println!("  DROP TABLE name");
        println!("  INSERT INTO table (cols) VALUES (vals)");
        println!("  SELECT cols FROM table [WHERE ...] [ORDER BY ...] [LIMIT ...]");
        println!("  UPDATE table SET col=val [WHERE ...]");
        println!("  DELETE FROM table [WHERE ...]");
        println!("  BEGIN / COMMIT / ROLLBACK");
        println!();
    }

    fn list_tables(&self) -> anyhow::Result<()> {
        let tables = self.db.tables()?;
        if tables.is_empty() {
            println!("No tables found.");
        } else {
            println!("Tables:");
            for table in tables {
                println!("  {}", table);
            }
        }
        Ok(())
    }

    fn describe_table(&self, name: &str) -> anyhow::Result<()> {
        match self.db.table_schema(name)? {
            Some(schema) => {
                println!("Table: {}", schema.name);
                if let Some(pk) = &schema.primary_key {
                    println!("Primary Key: {}", pk);
                }
                println!("\nColumns:");
                println!("{:<20} {:<15} {:<10}", "Name", "Type", "Nullable");
                println!("{:-<20} {:-<15} {:-<10}", "", "", "");
                for col in &schema.columns {
                    let nullable = if col.is_nullable() { "YES" } else { "NO" };
                    println!(
                        "{:<20} {:<15} {:<10}",
                        col.name,
                        format!("{:?}", col.data_type),
                        nullable
                    );
                }
            }
            None => {
                println!("Table '{}' not found.", name);
            }
        }
        Ok(())
    }

    fn print_stats(&self) {
        let stats = self.db.stats();
        println!("Database Statistics:");
        println!("  Tables: {}", stats.tables);
        println!("  Total Rows: {}", stats.total_rows);
        println!("  Size: {} bytes", stats.total_size_bytes);
        println!("  Active Transactions: {}", stats.active_transactions);
    }

    fn print_history(&self) {
        println!("Command History:");
        for (i, cmd) in self.history.iter().enumerate() {
            println!("  {}: {}", i + 1, cmd);
        }
    }

    fn print_result(&self, result: &QueryResult) {
        match result {
            QueryResult::Success { message } => {
                println!("{}", message);
            }
            QueryResult::Modified { rows_affected } => {
                println!("{} row(s) modified", rows_affected);
            }
            QueryResult::Select(rs) => {
                self.print_result_set(rs);
            }
            QueryResult::Transaction { message } => {
                println!("{}", message);
            }
        }
    }

    fn print_result_set(&self, rs: &crate::executor::ResultSet) {
        if rs.is_empty() {
            println!("(0 rows)");
            return;
        }

        // Get column names from first row.
        let columns: Vec<&String> = rs.rows[0].keys().collect();

        // Calculate column widths.
        let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
        for row in &rs.rows {
            for (i, col) in columns.iter().enumerate() {
                if let Some(val) = row.get(*col) {
                    let len = format_value(val).len();
                    widths[i] = widths[i].max(len);
                }
            }
        }

        // Print header.
        let header: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, c)| format!("{:width$}", c, width = widths[i]))
            .collect();
        println!("{}", header.join(" | "));

        // Print separator.
        let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        println!("{}", sep.join("-+-"));

        // Print rows.
        let limit = rs.len().min(self.config.max_rows);
        for row in rs.rows.iter().take(limit) {
            let values: Vec<String> = columns
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    let val = row.get(*col).map(format_value).unwrap_or_default();
                    format!("{:width$}", val, width = widths[i])
                })
                .collect();
            println!("{}", values.join(" | "));
        }

        // Print row count.
        if rs.len() > limit {
            println!("... ({} more rows)", rs.len() - limit);
        }
        println!("({} rows)", rs.len());
    }
}

fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(a) => format!("{:?}", a),
        serde_json::Value::Object(o) => format!("{:?}", o),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&serde_json::Value::Null), "NULL");
        assert_eq!(format_value(&serde_json::json!(true)), "true");
        assert_eq!(format_value(&serde_json::json!(42)), "42");
        assert_eq!(format_value(&serde_json::json!("hello")), "hello");
    }
}
