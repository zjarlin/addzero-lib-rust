//! Expression evaluation.

use serde_json::Value;

use super::error::{ExecuteError, ExecuteResult};
use crate::sql::{BinaryOperator, Expr, UnaryOperator};

#[cfg(test)]
use crate::sql::LiteralValue;

/// Evaluate an expression against a row.
pub fn evaluate(expr: &Expr, row: &serde_json::Map<String, Value>) -> ExecuteResult<Value> {
    match expr {
        Expr::Column(name) => row
            .get(name)
            .cloned()
            .ok_or_else(|| ExecuteError::ColumnNotFound(name.clone())),

        Expr::Literal(lit) => Ok(lit.to_json()),

        Expr::BinaryOp { left, op, right } => {
            let l = evaluate(left, row)?;
            let r = evaluate(right, row)?;
            eval_binary_op(&l, *op, &r)
        }

        Expr::UnaryOp { op, expr } => {
            let v = evaluate(expr, row)?;
            eval_unary_op(*op, &v)
        }

        Expr::IsNull { expr, negated } => {
            let v = evaluate(expr, row)?;
            let is_null = v.is_null();
            Ok(Value::Bool(if *negated { !is_null } else { is_null }))
        }

        Expr::InList {
            expr,
            list,
            negated,
        } => {
            let v = evaluate(expr, row)?;
            let in_list = list.iter().any(|item| {
                evaluate(item, row)
                    .map(|iv| values_equal(&v, &iv))
                    .unwrap_or(false)
            });
            Ok(Value::Bool(if *negated { !in_list } else { in_list }))
        }

        Expr::Between {
            expr,
            low,
            high,
            negated,
        } => {
            let v = evaluate(expr, row)?;
            let l = evaluate(low, row)?;
            let h = evaluate(high, row)?;
            let in_range = compare_values(&v, &l).map(|c| c >= 0).unwrap_or(false)
                && compare_values(&v, &h).map(|c| c <= 0).unwrap_or(false);
            Ok(Value::Bool(if *negated { !in_range } else { in_range }))
        }

        Expr::Like {
            expr,
            pattern,
            negated,
        } => {
            let v = evaluate(expr, row)?;
            let s = v.as_str().unwrap_or("");
            let matches = like_match(s, pattern);
            Ok(Value::Bool(if *negated { !matches } else { matches }))
        }

        Expr::Function { name, args } => {
            let evaluated: Vec<Value> = args
                .iter()
                .map(|a| evaluate(a, row))
                .collect::<ExecuteResult<_>>()?;
            eval_function(name, &evaluated)
        }

        Expr::Nested(inner) => evaluate(inner, row),
    }
}

/// Evaluate a WHERE clause, returning true if row matches.
pub fn matches_where(expr: &Expr, row: &serde_json::Map<String, Value>) -> ExecuteResult<bool> {
    let result = evaluate(expr, row)?;
    Ok(value_to_bool(&result))
}

fn eval_binary_op(left: &Value, op: BinaryOperator, right: &Value) -> ExecuteResult<Value> {
    match op {
        // Comparison operators
        BinaryOperator::Eq => Ok(Value::Bool(values_equal(left, right))),
        BinaryOperator::NotEq => Ok(Value::Bool(!values_equal(left, right))),
        BinaryOperator::Lt => Ok(Value::Bool(
            compare_values(left, right).map(|c| c < 0).unwrap_or(false),
        )),
        BinaryOperator::LtEq => Ok(Value::Bool(
            compare_values(left, right).map(|c| c <= 0).unwrap_or(false),
        )),
        BinaryOperator::Gt => Ok(Value::Bool(
            compare_values(left, right).map(|c| c > 0).unwrap_or(false),
        )),
        BinaryOperator::GtEq => Ok(Value::Bool(
            compare_values(left, right).map(|c| c >= 0).unwrap_or(false),
        )),

        // Logical operators
        BinaryOperator::And => Ok(Value::Bool(value_to_bool(left) && value_to_bool(right))),
        BinaryOperator::Or => Ok(Value::Bool(value_to_bool(left) || value_to_bool(right))),

        // Arithmetic operators
        BinaryOperator::Plus => eval_arithmetic(left, right, |a, b| a + b),
        BinaryOperator::Minus => eval_arithmetic(left, right, |a, b| a - b),
        BinaryOperator::Multiply => eval_arithmetic(left, right, |a, b| a * b),
        BinaryOperator::Divide => {
            let r = value_to_f64(right);
            if r == 0.0 {
                Err(ExecuteError::DivisionByZero)
            } else {
                eval_arithmetic(left, right, |a, b| a / b)
            }
        }
        BinaryOperator::Modulo => eval_arithmetic(left, right, |a, b| a % b),

        // String concatenation
        BinaryOperator::Concat => {
            let l = value_to_string(left);
            let r = value_to_string(right);
            Ok(Value::String(format!("{}{}", l, r)))
        }
    }
}

fn eval_unary_op(op: UnaryOperator, value: &Value) -> ExecuteResult<Value> {
    match op {
        UnaryOperator::Not => Ok(Value::Bool(!value_to_bool(value))),
        UnaryOperator::Minus => {
            let n = value_to_f64(value);
            if n.fract() == 0.0 {
                Ok(Value::Number((-n as i64).into()))
            } else {
                Ok(serde_json::Number::from_f64(-n)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            }
        }
        UnaryOperator::Plus => Ok(value.clone()),
    }
}

fn eval_arithmetic<F>(left: &Value, right: &Value, f: F) -> ExecuteResult<Value>
where
    F: Fn(f64, f64) -> f64,
{
    let l = value_to_f64(left);
    let r = value_to_f64(right);
    let result = f(l, r);

    // Return integer if both inputs were integers and result is whole
    if left.is_i64() && right.is_i64() && result.fract() == 0.0 {
        Ok(Value::Number((result as i64).into()))
    } else {
        Ok(serde_json::Number::from_f64(result)
            .map(Value::Number)
            .unwrap_or(Value::Null))
    }
}

fn eval_function(name: &str, args: &[Value]) -> ExecuteResult<Value> {
    let lower_name = name.to_lowercase();
    match lower_name.as_str() {
        "count" => Ok(Value::Number(1.into())), // Counting is done at aggregate level
        "lower" => {
            let s = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value::String(s.to_lowercase()))
        }
        "upper" => {
            let s = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value::String(s.to_uppercase()))
        }
        "length" | "len" => {
            let s = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(Value::Number(s.len().into()))
        }
        "coalesce" => {
            for arg in args {
                if !arg.is_null() {
                    return Ok(arg.clone());
                }
            }
            Ok(Value::Null)
        }
        "now" | "current_timestamp" => {
            let now = chrono::Utc::now().to_rfc3339();
            Ok(Value::String(now))
        }
        _ => Err(ExecuteError::InvalidExpression(format!(
            "unknown function: {}",
            name
        ))),
    }
}

/// Check if two JSON values are equal.
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => {
            // Compare as f64 for numeric equality
            a.as_f64()
                .zip(b.as_f64())
                .map(|(x, y)| (x - y).abs() < f64::EPSILON)
                .unwrap_or(false)
        }
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

/// Compare two JSON values, returning ordering.
fn compare_values(a: &Value, b: &Value) -> Option<i32> {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => {
            let a = a.as_f64()?;
            let b = b.as_f64()?;
            Some(a.partial_cmp(&b).map(|o| o as i32).unwrap_or(0))
        }
        (Value::String(a), Value::String(b)) => Some(a.cmp(b) as i32),
        (Value::Bool(a), Value::Bool(b)) => Some((*a as i32) - (*b as i32)),
        _ => None,
    }
}

/// Convert JSON value to boolean.
fn value_to_bool(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Convert JSON value to f64.
fn value_to_f64(v: &Value) -> f64 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        Value::String(s) => s.parse().unwrap_or(0.0),
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// Convert JSON value to string.
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        _ => v.to_string(),
    }
}

/// LIKE pattern matching.
fn like_match(s: &str, pattern: &str) -> bool {
    // Convert SQL LIKE pattern to regex-like matching
    // % = .* (any sequence)
    // _ = . (single char)
    let mut chars = pattern.chars().peekable();
    let mut regex_pattern = String::from("^");

    while let Some(c) = chars.next() {
        match c {
            '%' => regex_pattern.push_str(".*"),
            '_' => regex_pattern.push('.'),
            c => {
                if c.is_alphanumeric() || c == ' ' {
                    regex_pattern.push(c);
                } else {
                    regex_pattern.push('\\');
                    regex_pattern.push(c);
                }
            }
        }
    }
    regex_pattern.push('$');

    // Simple implementation without regex crate
    simple_like_match(s, pattern)
}

/// Simple LIKE matching without regex.
fn simple_like_match(s: &str, pattern: &str) -> bool {
    let s_chars: Vec<char> = s.chars().collect();
    let p_chars: Vec<char> = pattern.chars().collect();
    match_like(&s_chars, &p_chars, 0, 0)
}

fn match_like(s: &[char], p: &[char], si: usize, pi: usize) -> bool {
    if pi >= p.len() {
        return si >= s.len();
    }

    match p[pi] {
        '%' => {
            // Match any sequence (including empty)
            for i in si..=s.len() {
                if match_like(s, p, i, pi + 1) {
                    return true;
                }
            }
            false
        }
        '_' => {
            // Match single character
            si < s.len() && match_like(s, p, si + 1, pi + 1)
        }
        c => {
            // Match exact character (case-insensitive for SQL)
            si < s.len()
                && s[si].to_lowercase().eq(c.to_lowercase())
                && match_like(s, p, si + 1, pi + 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_row() -> serde_json::Map<String, Value> {
        let obj = json!({
            "id": 1,
            "name": "Alice",
            "age": 30,
            "active": true
        });
        obj.as_object().unwrap().clone()
    }

    #[test]
    fn test_column_eval() {
        let row = make_row();
        let expr = Expr::Column("name".into());
        assert_eq!(evaluate(&expr, &row).unwrap(), json!("Alice"));
    }

    #[test]
    fn test_literal_eval() {
        let row = make_row();
        let expr = Expr::Literal(LiteralValue::Integer(42));
        assert_eq!(evaluate(&expr, &row).unwrap(), json!(42));
    }

    #[test]
    fn test_comparison() {
        let row = make_row();
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Column("age".into())),
            op: BinaryOperator::Gt,
            right: Box::new(Expr::Literal(LiteralValue::Integer(25))),
        };
        assert_eq!(evaluate(&expr, &row).unwrap(), json!(true));
    }

    #[test]
    fn test_and_or() {
        let row = make_row();
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Column("active".into())),
            op: BinaryOperator::And,
            right: Box::new(Expr::BinaryOp {
                left: Box::new(Expr::Column("age".into())),
                op: BinaryOperator::Lt,
                right: Box::new(Expr::Literal(LiteralValue::Integer(50))),
            }),
        };
        assert_eq!(evaluate(&expr, &row).unwrap(), json!(true));
    }

    #[test]
    fn test_like_pattern() {
        assert!(simple_like_match("Alice", "A%"));
        assert!(simple_like_match("Alice", "%ice"));
        assert!(simple_like_match("Alice", "%lic%"));
        assert!(simple_like_match("Alice", "A____"));
        assert!(!simple_like_match("Alice", "B%"));
    }

    #[test]
    fn test_arithmetic() {
        let row = make_row();
        let expr = Expr::BinaryOp {
            left: Box::new(Expr::Column("age".into())),
            op: BinaryOperator::Plus,
            right: Box::new(Expr::Literal(LiteralValue::Integer(10))),
        };
        assert_eq!(evaluate(&expr, &row).unwrap(), json!(40));
    }

    #[test]
    fn test_in_list() {
        let row = make_row();
        let expr = Expr::InList {
            expr: Box::new(Expr::Column("name".into())),
            list: vec![
                Expr::Literal(LiteralValue::String("Alice".into())),
                Expr::Literal(LiteralValue::String("Bob".into())),
            ],
            negated: false,
        };
        assert_eq!(evaluate(&expr, &row).unwrap(), json!(true));
    }
}
