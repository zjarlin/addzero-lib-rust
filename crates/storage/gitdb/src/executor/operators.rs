//! 查询执行器的 Volcano/Iterator 风格算子。
//!
//! 每个算子都实现拉取式迭代模型：上层从根节点请求下一行，
//! 数据按需从下游算子逐层向上返回。

use serde_json::Value;
use std::collections::BTreeMap;

use super::error::ExecuteResult;
use super::eval::{evaluate, matches_where};
use crate::sql::{Expr, OrderBy, SelectColumn};

/// 查询执行管线中的一行数据。
pub type Row = BTreeMap<String, Value>;

/// 所有查询算子的统一接口。
pub trait Operator: Send {
    /// 拉取下一行；数据耗尽时返回 `None`。
    fn next_row(&mut self) -> ExecuteResult<Option<Row>>;

    /// 重置算子，使后续读取从头开始。
    fn reset(&mut self) -> ExecuteResult<()>;
}

/// 扫描算子，从表中顺序读取所有行。
pub struct ScanOperator {
    rows: Vec<Row>,
    position: usize,
}

impl ScanOperator {
    pub fn new(rows: Vec<Row>) -> Self {
        Self { rows, position: 0 }
    }
}

impl Operator for ScanOperator {
    fn next_row(&mut self) -> ExecuteResult<Option<Row>> {
        if self.position < self.rows.len() {
            let row = self.rows[self.position].clone();
            self.position += 1;
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) -> ExecuteResult<()> {
        self.position = 0;
        Ok(())
    }
}

/// 过滤算子，应用 `WHERE` 条件。
pub struct FilterOperator {
    source: Box<dyn Operator>,
    predicate: Expr,
}

impl FilterOperator {
    pub fn new(source: Box<dyn Operator>, predicate: Expr) -> Self {
        Self { source, predicate }
    }
}

impl Operator for FilterOperator {
    fn next_row(&mut self) -> ExecuteResult<Option<Row>> {
        loop {
            match self.source.next_row()? {
                Some(row) => {
                    let row_map = row.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                    if matches_where(&self.predicate, &row_map)? {
                        return Ok(Some(row));
                    }
                    // Row doesn't match, continue to next
                }
                None => return Ok(None),
            }
        }
    }

    fn reset(&mut self) -> ExecuteResult<()> {
        self.source.reset()
    }
}

/// 投影算子，选择列或计算表达式。
pub struct ProjectOperator {
    source: Box<dyn Operator>,
    columns: Vec<SelectColumn>,
}

impl ProjectOperator {
    pub fn new(source: Box<dyn Operator>, columns: Vec<SelectColumn>) -> Self {
        Self { source, columns }
    }
}

impl Operator for ProjectOperator {
    fn next_row(&mut self) -> ExecuteResult<Option<Row>> {
        match self.source.next_row()? {
            Some(row) => {
                let row_map: serde_json::Map<String, Value> =
                    row.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

                let mut projected = Row::new();
                for col in &self.columns {
                    match col {
                        SelectColumn::Wildcard => {
                            // Copy all columns
                            for (k, v) in &row {
                                projected.insert(k.clone(), v.clone());
                            }
                        }
                        SelectColumn::Column(name) => {
                            if let Some(v) = row.get(name) {
                                projected.insert(name.clone(), v.clone());
                            }
                        }
                        SelectColumn::Expr { expr, alias } => {
                            let value = evaluate(expr, &row_map)?;
                            let name = alias.clone().unwrap_or_else(|| format!("{:?}", expr));
                            projected.insert(name, value);
                        }
                    }
                }
                Ok(Some(projected))
            }
            None => Ok(None),
        }
    }

    fn reset(&mut self) -> ExecuteResult<()> {
        self.source.reset()
    }
}

/// 排序算子，按 `ORDER BY` 规则重排行。
pub struct SortOperator {
    source: Box<dyn Operator>,
    order_by: Vec<OrderBy>,
    sorted_rows: Option<Vec<Row>>,
    position: usize,
}

impl SortOperator {
    pub fn new(source: Box<dyn Operator>, order_by: Vec<OrderBy>) -> Self {
        Self {
            source,
            order_by,
            sorted_rows: None,
            position: 0,
        }
    }

    fn materialize(&mut self) -> ExecuteResult<()> {
        if self.sorted_rows.is_some() {
            return Ok(());
        }

        let mut rows = Vec::new();
        while let Some(row) = self.source.next_row()? {
            rows.push(row);
        }

        // Sort by order_by columns
        let order_by = self.order_by.clone();
        rows.sort_by(|a, b| {
            for ob in &order_by {
                let va = a.get(&ob.column);
                let vb = b.get(&ob.column);
                let cmp = compare_json_values(va, vb);
                if cmp != std::cmp::Ordering::Equal {
                    return if ob.ascending { cmp } else { cmp.reverse() };
                }
            }
            std::cmp::Ordering::Equal
        });

        self.sorted_rows = Some(rows);
        Ok(())
    }
}

impl Operator for SortOperator {
    fn next_row(&mut self) -> ExecuteResult<Option<Row>> {
        self.materialize()?;

        if let Some(ref rows) = self.sorted_rows {
            if self.position < rows.len() {
                let row = rows[self.position].clone();
                self.position += 1;
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    fn reset(&mut self) -> ExecuteResult<()> {
        self.source.reset()?;
        self.sorted_rows = None;
        self.position = 0;
        Ok(())
    }
}

/// 限制算子，处理 `LIMIT` 与 `OFFSET`。
pub struct LimitOperator {
    source: Box<dyn Operator>,
    limit: usize,
    offset: usize,
    current: usize,
    skipped: usize,
}

impl LimitOperator {
    pub fn new(source: Box<dyn Operator>, limit: usize, offset: usize) -> Self {
        Self {
            source,
            limit,
            offset,
            current: 0,
            skipped: 0,
        }
    }
}

impl Operator for LimitOperator {
    fn next_row(&mut self) -> ExecuteResult<Option<Row>> {
        // Skip offset rows
        while self.skipped < self.offset {
            if self.source.next_row()?.is_none() {
                return Ok(None);
            }
            self.skipped += 1;
        }

        // Return up to limit rows
        if self.current < self.limit {
            if let Some(row) = self.source.next_row()? {
                self.current += 1;
                return Ok(Some(row));
            }
        }
        Ok(None)
    }

    fn reset(&mut self) -> ExecuteResult<()> {
        self.source.reset()?;
        self.current = 0;
        self.skipped = 0;
        Ok(())
    }
}

/// 按 GitDB 当前规则比较两个 JSON 值的排序位置。
fn compare_json_values(a: Option<&Value>, b: Option<&Value>) -> std::cmp::Ordering {
    match (a, b) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(a), Some(b)) => match (a, b) {
            (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
            (Value::Null, _) => std::cmp::Ordering::Less,
            (_, Value::Null) => std::cmp::Ordering::Greater,
            (Value::Number(a), Value::Number(b)) => {
                let a = a.as_f64().unwrap_or(0.0);
                let b = b.as_f64().unwrap_or(0.0);
                a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        },
    }
}
