# GitDB

> **A Git backed document database That Absolutely Nobody Asked For**

[![Tests](https://img.shields.io/badge/tests-134%20passing-brightgreen.svg)]()

---

## What Is This?

GitDB is a fully-functional SQL database that stores all its data in a Git repository. Every INSERT is a commit. Every table is a directory. Your entire database history is preserved forever in `.git/`.

**Why would anyone build this?** Great question. I don't have a good answer either.

**Should you use this in production?** Absolutely! why not ( ͡° ͜ʖ ͡°).

> Demo - https://github.com/qeqqe/chat-app-with-gitdb

## Features

- **Full SQL Support** - CREATE, INSERT, SELECT, UPDATE, DELETE, the whole demn shebang
- **Git-Native Storage** - Every mutation is a commit, every table is a tree
- **ACID Transactions** - BEGIN, COMMIT, ROLLBACK with proper isolation
- **Built-in Version History** - It's git ofc..
- **Query Planning** - Cost-based optimizer because we're not savages
- **Interactive REPL** - Pretty terminal interface for your hacking pleasure
- **Connection Pooling** - For when you need to pretend this is enterprise-ready

---

## Installation

### As a CLI Tool (install globally)

```bash
# clone this
git clone https://github.com/qeqqe/gitdb.git
cd gitdb

# install globally
cargo install --path .

# check if it works
gitdb --version
```

### As a Library in Your Project

Add this to your `Cargo.toml` (will be added to crates.io soon):

```toml
[dependencies]
gitdb = { git = "https://github.com/qeqqe/gitdb.git" }
```

Or if you want to use a local path:

```toml
[dependencies]
gitdb = { path = "../path/to/gitdb" }
```

Then in your Rust code:

```rust
use gitdb::db::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Database::open("./my_database")?;
    db.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")?;
    db.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')")?;
    
    let result = db.execute("SELECT * FROM users")?;
    println!("{:?}", result);
    Ok(())
}
```

### Build from Source (without installing)

```bash
git clone https://github.com/qeqqe/gitdb.git
cd gitdb
cargo build --release
./target/release/gitdb --help
```

---

## CLI Usage

### Basic Commands

```bash
# start the interactive REPL (default database: .gitdb)
gitdb

# Use a specific database directory
gitdb mydb
gitdb -d /path/to/my/database

# Run a single query and exit
gitdb -e "SELECT * FROM users"
gitdb -d mydb -e "INSERT INTO users (id, name) VALUES ('1', 'Bob')"

# verbose mode (see what's happening under the hood)
gitdb -v

# help
gitdb --help

# version
gitdb --version
```

### CLI Flags Reference

| Flag | Long Form | Description |
|------|-----------|-------------|
| `-d` | `--database PATH` | Path to database directory (default: `.gitdb`) |
| `-e` | `--execute SQL` | Execute SQL statement and exit |
| `-v` | `--verbose` | Enable verbose output |
| `-h` | `--help` | Show help message |
| | `--version` | Show version |

---

## REPL Commands

Once you're in the REPL, you've got these commands at your disposal:

| Command | Aliases | Description |
|---------|---------|-------------|
| `.help` | `.h`, `.?` | Show help message |
| `.quit` | `.exit`, `.q` | Get the hell out |
| `.tables` | `.dt` | List all tables |
| `.schema <table>` | `.describe`, `.d` | Show table schema |
| `.stats` | | Show database statistics |
| `.history` | | Show command history |
| `.explain <sql>` | | Show query execution plan |
| `.timing` | | Toggle timing display |
| `.clear` | | Clear the screen |

### REPL Example Session

```
╔═══════════════════════════════════════════════════════════════╗
║                        GitDB v0.1.0                           ║
║            A Git-backed Document Database                     ║
╠═══════════════════════════════════════════════════════════════╣
║   Type .help for commands, or enter SQL statements            ║
╚═══════════════════════════════════════════════════════════════╝

gitdb> CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT, email TEXT);
Created table 'users'
Time: 12.345ms

gitdb> INSERT INTO users (id, name, email) VALUES ('1', 'Alice', 'alice@example.com');
1 row(s) modified
Time: 8.234ms

gitdb> SELECT * FROM users;
id | name  | email
---+-------+------------------
1  | Alice | alice@example.com
(1 rows)
Time: 2.456ms

gitdb> .tables
Tables:
  users

gitdb> .schema users
Table: users
Primary Key: id

Columns:
Name                 Type            Nullable
-------------------- --------------- ----------
id                   Text            NO
name                 Text            YES
email                Text            YES

gitdb> .quit
Goodbye!
```

---

## SQL Reference

### Supported SQL Statements

#### CREATE TABLE
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT,
    age INTEGER,
    active BOOLEAN
);
```

Supported data types: `TEXT`, `INTEGER`, `REAL`, `BOOLEAN`, `BLOB`

#### DROP TABLE
```sql
DROP TABLE users;
```

#### INSERT
```sql
-- single row
INSERT INTO users (id, name, age) VALUES ('1', 'Alice', 30);

-- the basics, nothing fancy
INSERT INTO products (sku, name, price) VALUES ('ABC123', 'Widget', 19.99);
```

#### SELECT
```sql
-- Select all columns
SELECT * FROM users;

-- Select specific columns
SELECT name, email FROM users;

-- With WHERE clause
SELECT * FROM users WHERE age > 21;

-- With ORDER BY
SELECT * FROM users ORDER BY name ASC;

-- With LIMIT
SELECT * FROM users LIMIT 10;

-- Complex conditions
SELECT * FROM users WHERE age > 21 AND active = true;
```

#### UPDATE
```sql
UPDATE users SET name = 'Bob' WHERE id = '1';
UPDATE products SET price = 29.99 WHERE sku = 'ABC123';
```

#### DELETE
```sql
DELETE FROM users WHERE id = '1';
DELETE FROM products WHERE price < 10;
```

#### Transactions
```sql
BEGIN;
INSERT INTO accounts (id, balance) VALUES ('1', 1000);
INSERT INTO accounts (id, balance) VALUES ('2', 500);
COMMIT;

-- Or roll it back if shit goes wrong
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE id = '1';
ROLLBACK;  -- Nope, nevermind
```

---

## Rust API Usage

### Basic Usage

```rust
use GitDB::db::{Database, DatabaseConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open or create a database
    let mut db = Database::open("./my_database")?;
    
    // Create a table
    db.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")?;
    
    // Insert some data
    db.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')")?;
    db.execute("INSERT INTO users (id, name) VALUES ('2', 'Bob')")?;
    
    // Query that shit
    let result = db.execute("SELECT * FROM users WHERE name = 'Alice'")?;
    println!("{:?}", result);
    
    Ok(())
}
```

### With Configuration

```rust
use GitDB::db::{Database, DatabaseConfig};

let config = DatabaseConfig::new("./my_database")
    .create_if_missing(true)
    .verbose(true)
    .auto_commit(true);

let mut db = Database::open_with_config(config)?;
```

### Batch Execution

```rust
// Execute multiple statements at once
let results = db.execute_batch(r#"
    CREATE TABLE products (sku TEXT PRIMARY KEY, name TEXT, price REAL);
    INSERT INTO products (sku, name, price) VALUES ('A1', 'Widget', 9.99);
    INSERT INTO products (sku, name, price) VALUES ('A2', 'Gadget', 19.99);
    SELECT * FROM products;
"#)?;
```

### Query Planning & Explain

```rust
// See what the query planner is thinking
let plan = db.explain("SELECT * FROM users WHERE id = '1'")?;
println!("{}", plan);
```

### Database Statistics

```rust
let stats = db.stats();
println!("Tables: {}", stats.tables);
println!("Total Rows: {}", stats.total_rows);
println!("Active Transactions: {}", stats.active_transactions);
```

### Table Operations

```rust
// List all tables
let tables = db.tables()?;

// Check if table exists
if db.table_exists("users") {
    println!("Users table exists!");
}

// Get table schema
if let Some(schema) = db.table_schema("users")? {
    println!("Table: {}", schema.name);
    for col in &schema.columns {
        println!("  {} {:?}", col.name, col.data_type);
    }
}
```

### Transaction API

```rust
// Manual transaction control
db.execute("BEGIN")?;
db.execute("INSERT INTO users (id, name) VALUES ('3', 'Charlie')")?;
db.execute("COMMIT")?;

// Or use the closure-based API
db.transaction(|db| {
    db.execute("INSERT INTO users (id, name) VALUES ('4', 'Dave')")?;
    db.execute("INSERT INTO users (id, name) VALUES ('5', 'Eve')")?;
    Ok(())
})?;
```

### Version History

```rust
// Get commit history (it's Git, baby!)
let history = db.history(Some(10))?;  // Last 10 commits
for commit in history {
    println!("{}: {} ({})", commit.id, commit.message, commit.timestamp);
}

// create a snapshot
let snapshot_id = db.snapshot("Before the big migration")?;
```

### Connection Pooling

```rust
use GitDB::db::{ConnectionPool, DatabaseConfig};

// create a pool with max 10 connections
let pool = ConnectionPool::new(DatabaseConfig::new("./mydb"), 10)?;

// get a connection
let mut conn = pool.get()?;
conn.execute("SELECT * FROM users")?;

// connection automatically returns to pool when dropped
```

---

## Architecture

if for some weird reason you're curious about this dumpster fire:

```
src/
├── storage/          # Git integration layer
│   ├── repository.rs # GitRepository wrapper
│   ├── tree.rs       # Tree operations (tables/rows)
│   ├── commit.rs     # Commit operations
│   ├── refs.rs       # Branch/ref management
│   ├── blob.rs       # Blob serialization
│   └── types.rs      # Core types (RowKey, TableName, etc.)
├── transaction/      # ACID transaction support
│   ├── manager.rs    # Transaction lifecycle
│   ├── context.rs    # Transaction state (typestate pattern)
│   └── isolation.rs  # Isolation levels
├── catalog/          # Schema management
│   ├── schema.rs     # TableSchema, ColumnDef
│   └── catalog.rs    # Schema storage in _schema/
├── sql/              # SQL parsing
│   ├── parser.rs     # sqlparser integration
│   ├── ast.rs        # Our AST types
│   └── types.rs      # SQL types
├── executor/         # Query execution
│   ├── executor.rs   # QueryExecutor
│   ├── operators.rs  # Volcano-model operators
│   └── eval.rs       # Expression evaluation
├── planner/          # Query planning
│   ├── logical.rs    # Logical plan
│   ├── physical.rs   # Physical plan
│   ├── optimizer.rs  # Cost-based optimizer
│   └── planner.rs    # Plan generation
├── db/               # High-level API
│   ├── api.rs        # Database struct
│   ├── connection.rs # Connection pooling
│   └── repl.rs       # Interactive REPL
└── main.rs           # CLI entry point
```

---

## How It Actually Works

1. **Tables are directories** - Each table is a directory under the Git tree
2. **Rows are JSON blobs** - Each row is a JSON file named by its primary key
3. **Mutations are commits** - Every INSERT/UPDATE/DELETE creates a Git commit
4. **Schemas live in `_schema/`** - Table definitions stored as JSON
5. **Transactions use branches** - Each transaction gets its own branch, merged on commit

So when you do:
```sql
INSERT INTO users (id, name) VALUES ('1', 'Alice');
```

GitDB literally creates a commit with:
- Tree: `users/1.json` containing `{"id": "1", "name": "Alice"}`
- Message: `INSERT INTO users`

It's beautifully stupid.

---

## Performance

It's very fast (only like 50x-60x (maybe more) times slower then pgsql):

| Operation | Performance |
|-----------|-------------|
| SELECT | Actually pretty fast |
| INSERT | One Git commit per row, so... yeah |
| UPDATE | Same shit |
| Bulk operations | Pain |
| vs. PostgreSQL | lmao |

---

## Contributing

1. Fork it
2. Create your feature branch (`git checkout -b feature/even-more-cursed`)
3. Write some tests (we're not animals)
4. Commit your changes (`git commit -am 'Add some cursed feature'`)
5. Push to the branch (`git push origin feature/even-more-cursed`)
6. Create a Pull Request

---

## Acknowledgments

- The Git maintainers, for creating a data structure we were never meant to abuse this way
- The Rust community, for making this cursed project actually reliable

---
> Note: While playing around with it i accidentally messed up and messed up the whole repo, this the new repo with all the commits squashed that i made in previous one.
---

<p align="center">
  <i>Built with 🦀 and questionable life choices</i>
</p>
