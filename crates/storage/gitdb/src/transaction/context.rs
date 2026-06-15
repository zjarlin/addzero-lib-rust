//! 使用 typestate 模式建模的事务上下文。
//!
//! `Transaction<State>` 把事务生命周期编码进类型参数：只有
//! `TxActive` 能执行读写操作，提交或回滚后的事务会移动到不可复用状态。
//! 这样可以在编译期挡住“提交后继续写入”一类误用。

use std::collections::BTreeMap;
use std::marker::PhantomData;

use az_derive_aliases::{apply, plain_clone_debug, plain_debug};
use serde_json::Value;

use crate::storage::{
    BranchName, CommitId, GitRepository, Row, RowKey, TableName, is_retriable,
};
use crate::transaction::isolation::IsolationLevel;

/// 活跃事务的状态标记。
#[apply(plain_debug)]
pub struct TxActive;

/// 已提交事务的状态标记。
#[apply(plain_debug)]
pub struct TxCommitted;

/// 已回滚事务的状态标记。
#[apply(plain_debug)]
pub struct TxAborted;

/// 事务管理器保存的事务元数据。
#[apply(plain_clone_debug)]
pub struct TransactionMetadata {
    /// 全局唯一的事务 ID。
    pub tx_id: String,
    /// 事务对应的 Git 分支名。
    pub branch: BranchName,
    /// 事务开始时的基准提交。
    pub base_commit: CommitId,
    /// 事务分支当前指向的提交。
    pub current_commit: CommitId,
    /// 本事务使用的隔离级别。
    pub isolation: IsolationLevel,
    /// 事务开始时间。
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// 带 typestate 生命周期约束的数据库事务。
///
/// `State` 参数表达事务当前阶段：`TxActive` 可继续读写，`TxCommitted`
/// 表示已经成功提交，`TxAborted` 表示已经回滚。后两者不再暴露写操作。
pub struct Transaction<State> {
    /// 事务元数据。
    pub(crate) metadata: TransactionMetadata,
    /// 事务操作的 Git 仓库句柄。
    pub(crate) repo: GitRepository,
    /// 用于表达 typestate 的零大小标记。
    _state: PhantomData<State>,
}

impl<State> Transaction<State> {
    /// 返回事务 ID。
    pub fn id(&self) -> &str {
        &self.metadata.tx_id
    }

    /// 返回事务开始时的基准提交。
    pub fn base_commit(&self) -> CommitId {
        self.metadata.base_commit
    }

    /// 返回事务隔离级别。
    pub fn isolation(&self) -> IsolationLevel {
        self.metadata.isolation
    }

    /// 返回事务分支名。
    pub fn branch(&self) -> &BranchName {
        &self.metadata.branch
    }
}

impl Transaction<TxActive> {
    /// 创建新的活跃事务。
    pub(crate) fn new(
        repo: GitRepository,
        tx_id: String,
        branch: BranchName,
        base_commit: CommitId,
        isolation: IsolationLevel,
    ) -> Self {
        Self {
            metadata: TransactionMetadata {
                tx_id,
                branch,
                base_commit,
                current_commit: base_commit,
                isolation,
                started_at: chrono::Utc::now(),
            },
            repo,
            _state: PhantomData,
        }
    }

    /// 根据隔离级别选择读取提交点。
    ///
    /// 当前实现始终从事务分支的当前提交读取，因此事务能读到自己的写入，
    /// 但不会看到并发事务已经提交到 `main` 的修改。`ReadCommitted` 与
    /// `RepeatableRead` 的语义差异后续应在这里收窄实现。
    fn read_commit(&self) -> anyhow::Result<CommitId> {
        // Always read from transaction's current state to see own writes
        Ok(self.metadata.current_commit)
    }

    /// 返回事务分支当前提交，写操作会基于它继续追加。
    pub fn current_commit(&self) -> CommitId {
        self.metadata.current_commit
    }

    // ==================== Table Operations ====================

    /// 在事务分支中创建新表。
    pub fn create_table(&mut self, table: &TableName) -> anyhow::Result<()> {
        let new_commit = self.repo.create_table(
            table,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// 在事务分支中删除表。
    pub fn drop_table(&mut self, table: &TableName) -> anyhow::Result<()> {
        let new_commit = self.repo.drop_table(
            table,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// 列出当前事务可见的所有表。
    pub fn list_tables(&self) -> anyhow::Result<Vec<TableName>> {
        let commit = self.read_commit()?;
        self.repo.list_tables(commit)
    }

    /// 检查当前事务视图中表是否存在。
    pub fn table_exists(&self, table: &TableName) -> anyhow::Result<bool> {
        let commit = self.read_commit()?;
        self.repo.table_exists(table, commit)
    }

    // ==================== Row Operations ====================

    /// 插入一行新数据。
    pub fn insert(&mut self, table: &TableName, row: Row) -> anyhow::Result<()> {
        let new_commit = self.repo.insert_row(
            table,
            row,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// 从原始列值插入一行。
    pub fn insert_data(
        &mut self,
        table: &TableName,
        key: RowKey,
        data: BTreeMap<String, Value>,
    ) -> anyhow::Result<()> {
        let row = Row::new(key, data);
        self.insert(table, row)
    }

    /// 更新已有行。
    pub fn update(&mut self, table: &TableName, row: Row) -> anyhow::Result<()> {
        let new_commit = self.repo.update_row(
            table,
            row,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// 插入或更新一行。
    pub fn upsert(&mut self, table: &TableName, row: Row) -> anyhow::Result<()> {
        let new_commit = self.repo.upsert_row(
            table,
            row,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// 删除一行。
    pub fn delete(&mut self, table: &TableName, key: &RowKey) -> anyhow::Result<()> {
        let new_commit = self.repo.delete_row(
            table,
            key,
            self.metadata.current_commit,
            Some(&self.metadata.tx_id),
        )?;
        self.metadata.current_commit = new_commit;
        self.update_branch()?;
        Ok(())
    }

    /// 读取单行。
    pub fn read(&self, table: &TableName, key: &RowKey) -> anyhow::Result<Option<Row>> {
        let commit = self.read_commit()?;
        self.repo.read_row(table, key, commit)
    }

    /// 扫描表内所有行。
    pub fn scan(&self, table: &TableName) -> anyhow::Result<Vec<Row>> {
        let commit = self.read_commit()?;
        self.repo.scan_table(table, commit)
    }

    /// 列出表内所有行键。
    pub fn list_keys(&self, table: &TableName) -> anyhow::Result<Vec<RowKey>> {
        let commit = self.read_commit()?;
        self.repo.list_rows(table, commit)
    }

    // ==================== Transaction Control ====================

    /// 将事务分支推进到当前提交。
    fn update_branch(&self) -> anyhow::Result<()> {
        self.repo
            .update_branch(&self.metadata.branch, self.metadata.current_commit)
    }

    /// 提交事务。
    ///
    /// 提交时尝试把 `main` 快进到事务分支。
    ///
    /// 如果 `main` 已被并发事务推进，会先做冲突检测；检测到冲突时删除事务分支并返回错误。
    pub fn commit(self) -> anyhow::Result<Transaction<TxCommitted>> {
        // Check for conflicts by seeing if main has moved
        let main_head = self.repo.head()?;

        if main_head != self.metadata.base_commit {
            // Main has moved - check for conflicts
            let conflicts = self
                .repo
                .detect_conflicts(&self.metadata.branch, main_head)?;
            if !conflicts.is_empty() {
                // Clean up the branch before returning error
                let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);
                anyhow::bail!("transaction conflict: {conflicts:?}");
            }
        }

        // Fast-forward main to our commit
        match self
            .repo
            .fast_forward_main(&self.metadata.branch, self.metadata.base_commit)
        {
            Ok(_) => {}
            Err(error) if is_retriable(&error) => {
                // Another transaction just committed - retry detection
                let main_head = self.repo.head()?;
                let conflicts = self
                    .repo
                    .detect_conflicts(&self.metadata.branch, main_head)?;
                let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);
                anyhow::bail!("transaction conflict: {conflicts:?}");
            }
            Err(error) => {
                let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);
                return Err(error);
            }
        }

        // Clean up the transaction branch
        let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);

        Ok(Transaction {
            metadata: self.metadata,
            repo: self.repo,
            _state: PhantomData,
        })
    }

    /// 回滚事务。
    ///
    /// 回滚通过删除事务分支丢弃本事务的所有改动。
    pub fn rollback(self) -> anyhow::Result<Transaction<TxAborted>> {
        // Clean up the transaction branch
        let _ = self.repo.delete_transaction_branch(&self.metadata.tx_id);

        Ok(Transaction {
            metadata: self.metadata,
            repo: self.repo,
            _state: PhantomData,
        })
    }
}

impl Transaction<TxCommitted> {
    /// 返回已提交事务的最终提交 ID。
    pub fn final_commit(&self) -> CommitId {
        self.metadata.current_commit
    }
}

impl Transaction<TxAborted> {
    /// 返回事务是否由回滚结束。
    pub fn was_rolled_back(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, GitRepository) {
        let dir = TempDir::new().unwrap();
        let repo = GitRepository::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_transaction_insert_read() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create a transaction
        let branch = repo.create_transaction_branch("tx001", head).unwrap();
        let mut tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head,
            IsolationLevel::ReadCommitted,
        );

        // Create table and insert row
        let table = TableName::new("users").unwrap();
        tx.create_table(&table).unwrap();

        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        tx.insert_data(&table, key.clone(), data).unwrap();

        // Read within transaction
        let row = tx.read(&table, &key).unwrap().unwrap();
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));

        // Commit
        let _committed = tx.commit().unwrap();

        // Verify on main
        let main_head = repo.head().unwrap();
        let row = repo.read_row(&table, &key, main_head).unwrap().unwrap();
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));
    }

    #[test]
    fn test_transaction_rollback() {
        let (_dir, repo) = setup();
        let initial_head = repo.head().unwrap();

        // Create table and update main branch to include it
        let table = TableName::new("users").unwrap();
        let head_with_table = repo.create_table(&table, initial_head, None).unwrap();
        // Update main branch to point to the new commit with the table
        let main_branch = BranchName::main();
        repo.update_branch(&main_branch, head_with_table).unwrap();

        // Start transaction from the commit that has the table
        let branch = repo
            .create_transaction_branch("tx001", head_with_table)
            .unwrap();
        let mut tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head_with_table,
            IsolationLevel::ReadCommitted,
        );

        // Insert row
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        tx.insert_data(&table, key.clone(), data).unwrap();

        // Rollback
        let _aborted = tx.rollback().unwrap();

        // Verify row doesn't exist on main (table exists, but row shouldn't)
        let main_head = repo.head().unwrap();
        let row = repo.read_row(&table, &key, main_head).unwrap();
        assert!(row.is_none());
    }

    #[test]
    fn test_isolation_levels() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        // Start transaction with ReadCommitted
        let branch = repo.create_transaction_branch("tx001", head).unwrap();
        let tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head,
            IsolationLevel::ReadCommitted,
        );

        // Insert row outside transaction (simulating another committed transaction)
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        let row = Row::new(key.clone(), data);
        let _ = repo.insert_row(&table, row, head, None).unwrap();

        // Transaction reads from its own branch (snapshot isolation in practice)
        // So it won't see the row inserted outside
        let row = tx.read(&table, &key).unwrap();
        assert!(row.is_none());

        tx.rollback().unwrap();
    }

    #[test]
    fn test_transaction_sees_own_writes() {
        let (_dir, repo) = setup();
        let head = repo.head().unwrap();

        // Create table
        let table = TableName::new("users").unwrap();
        let head = repo.create_table(&table, head, None).unwrap();

        // Start transaction
        let branch = repo.create_transaction_branch("tx001", head).unwrap();
        let mut tx = Transaction::<TxActive>::new(
            repo.clone(),
            "tx001".to_string(),
            branch,
            head,
            IsolationLevel::RepeatableRead,
        );

        // Insert a row in the transaction
        let key = RowKey::new("user1").unwrap();
        let mut data = BTreeMap::new();
        data.insert("name".to_string(), Value::String("Alice".to_string()));
        tx.insert_data(&table, key.clone(), data).unwrap();

        // Transaction should see its own writes
        let row = tx.read(&table, &key).unwrap();
        assert!(row.is_some());
        assert_eq!(
            row.unwrap().get("name"),
            Some(&Value::String("Alice".to_string()))
        );

        tx.rollback().unwrap();
    }
}
