use regex::Regex;
use serde_json::{json, Value};

use crate::payload_tools::types::{SqlQueryResult, ValidationRule};
use crate::payload_tools::validator::validation_rules;

#[derive(Debug)]
enum Query {
    Select {
        columns: Vec<String>,
        table: String,
        where_clause: Option<Condition>,
        order_by: Vec<OrderBy>,
        limit: Option<usize>,
    },
    Describe { table: String },
}

#[derive(Debug, Clone)]
enum Condition {
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Comparison { column: String, operator: Operator, value: Value },
}

#[derive(Debug, Clone, Copy)]
enum Operator {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    In,
}

#[derive(Debug, Clone)]
struct OrderBy {
    column: String,
    direction: SortDirection,
}

#[derive(Debug, Clone, Copy)]
enum SortDirection {
    Asc,
    Desc,
}

pub fn execute_sql_query(sql: &str) -> Result<SqlQueryResult, String> {
    let query = parse_query(sql)?;
    match query {
        Query::Select {
            columns,
            table,
            where_clause,
            order_by,
            limit,
        } => execute_select_query(columns, table, where_clause, order_by, limit),
        Query::Describe { table } => execute_describe_query(table),
    }
}

fn parse_query(sql: &str) -> Result<Query, String> {
    let trimmed = sql.trim();
    let select_re = Regex::new(
        r"(?i)^SELECT\s+(.*?)\s+FROM\s+(.*?)(?:\s+WHERE\s+(.*?))?(?:\s+ORDER\s+BY\s+(.*?))?(?:\s+LIMIT\s+(\d+))?$",
    )
    .map_err(|err| format!("Failed to compile SELECT regex: {err}"))?;

    if let Some(caps) = select_re.captures(trimmed) {
        let columns = caps
            .get(1)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .split(',')
            .map(|c| c.trim().to_string())
            .collect::<Vec<_>>();
        let table = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
        let where_clause = caps
            .get(3)
            .and_then(|m| parse_where_clause(m.as_str()).ok());
        let order_by = caps
            .get(4)
            .map(|m| parse_order_by_clause(m.as_str()))
            .unwrap_or_else(Vec::new);
        let limit = caps
            .get(5)
            .and_then(|m| m.as_str().parse::<usize>().ok());

        return Ok(Query::Select {
            columns,
            table: table.trim().to_string(),
            where_clause,
            order_by,
            limit,
        });
    }

    let describe_re =
        Regex::new(r"(?i)^DESCRIBE\s+(.*?)$").map_err(|err| format!("Regex error: {err}"))?;
    if let Some(caps) = describe_re.captures(trimmed) {
        let table = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
        return Ok(Query::Describe {
            table: table.trim().to_string(),
        });
    }

    Err("Unsupported query type. Only SELECT and DESCRIBE are supported.".to_string())
}

fn parse_where_clause(where_clause: &str) -> Result<Condition, String> {
    let and_parts = split_case_insensitive(where_clause, "AND");
    let mut and_conditions = Vec::new();

    for part in and_parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let or_parts = split_case_insensitive(&part, "OR");
        if or_parts.len() > 1 {
            let mut or_conditions = Vec::new();
            for or in or_parts {
                let or_trimmed = or.trim();
                if or_trimmed.is_empty() {
                    continue;
                }
                or_conditions.push(parse_condition(or_trimmed)?);
            }
            and_conditions.push(Condition::Or(or_conditions));
        } else {
            and_conditions.push(parse_condition(trimmed)?);
        }
    }

    if and_conditions.is_empty() {
        return Err("Failed to parse WHERE clause".to_string());
    }

    if and_conditions.len() == 1 {
        Ok(and_conditions.remove(0))
    } else {
        Ok(Condition::And(and_conditions))
    }
}

fn split_case_insensitive(input: &str, delimiter: &str) -> Vec<String> {
    let pattern = format!(r"(?i)\s*{}\s*", regex::escape(delimiter));
    let re = Regex::new(&pattern).unwrap();
    re.split(input).map(|s| s.to_string()).collect()
}

fn parse_condition(condition: &str) -> Result<Condition, String> {
    let re = Regex::new(r"(?i)\s*(.*?)\s*(=|!=|>|<|>=|<=|LIKE|IN)\s*(.*)\s*$")
        .map_err(|err| format!("Regex error: {err}"))?;
    let caps = re
        .captures(condition)
        .ok_or_else(|| format!("Invalid condition format: {condition}"))?;

    let column = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
    let operator = caps.get(2).map(|m| m.as_str()).unwrap_or("=");
    let value_raw = caps.get(3).map(|m| m.as_str().trim()).unwrap_or("");

    let operator = match operator.to_ascii_uppercase().as_str() {
        "=" => Operator::Eq,
        "!=" => Operator::Neq,
        ">" => Operator::Gt,
        "<" => Operator::Lt,
        ">=" => Operator::Gte,
        "<=" => Operator::Lte,
        "LIKE" => Operator::Like,
        "IN" => Operator::In,
        other => return Err(format!("Unsupported operator: {other}")),
    };

    let value = if matches!(operator, Operator::In) {
        let values = value_raw
            .trim_matches(|c| c == '(' || c == ')')
            .split(',')
            .filter_map(|v| parse_value(v.trim()).ok())
            .collect::<Vec<_>>();
        Value::Array(values)
    } else {
        parse_value(value_raw)?
    };

    Ok(Condition::Comparison {
        column,
        operator,
        value,
    })
}

fn parse_value(raw: &str) -> Result<Value, String> {
    if raw.starts_with('"') && raw.ends_with('"') || raw.starts_with('\'') && raw.ends_with('\'') {
        let trimmed = raw.trim_matches(|c| c == '"' || c == '\'');
        return Ok(Value::String(trimmed.to_string()));
    }

    if let Ok(num) = raw.parse::<f64>() {
        return Ok(Value::from(num));
    }

    if raw.eq_ignore_ascii_case("true") {
        return Ok(Value::Bool(true));
    }
    if raw.eq_ignore_ascii_case("false") {
        return Ok(Value::Bool(false));
    }
    if raw.eq_ignore_ascii_case("null") {
        return Ok(Value::Null);
    }

    Ok(Value::String(raw.to_string()))
}

fn parse_order_by_clause(order_by_clause: &str) -> Vec<OrderBy> {
    order_by_clause
        .split(',')
        .filter_map(|part| {
            let mut split = part.trim().split_whitespace();
            let column = split.next()?.to_string();
            let direction = split
                .next()
                .map(|dir| {
                    if dir.eq_ignore_ascii_case("DESC") {
                        SortDirection::Desc
                    } else {
                        SortDirection::Asc
                    }
                })
                .unwrap_or(SortDirection::Asc);
            Some(OrderBy { column, direction })
        })
        .collect()
}

fn execute_select_query(
    columns: Vec<String>,
    table: String,
    where_clause: Option<Condition>,
    order_by: Vec<OrderBy>,
    limit: Option<usize>,
) -> Result<SqlQueryResult, String> {
    let mut data: Vec<ValidationRule> = match table.to_ascii_lowercase().as_str() {
        "validation_rules" => validation_rules(),
        _ => return Err(format!("Unknown table: {table}")),
    };

    if let Some(condition) = where_clause {
        data.retain(|item| evaluate_where_clause(item, &condition));
    }

    if !order_by.is_empty() {
        sort_data(&mut data, &order_by);
    }

    if let Some(limit) = limit {
        data.truncate(limit);
    }

    let select_all = columns.iter().any(|c| c == "*");
    let rows = data
        .into_iter()
        .map(|item| {
            if select_all {
                serde_json::to_value(&item).unwrap_or_else(|_| Value::Null)
            } else {
                let mut map = serde_json::Map::new();
                for column in &columns {
                    let value = match column.as_str() {
                        "id" => Value::String(item.id.clone()),
                        "name" => Value::String(item.name.clone()),
                        "description" => Value::String(item.description.clone()),
                        "category" => Value::String(item.category.clone()),
                        "fileTypes" | "file_types" => json!(
                            item.file_types.iter().map(|ft| ft.as_str()).collect::<Vec<_>>()
                        ),
                        "examples" => json!({
                            "valid": item.examples.valid,
                            "invalid": item.examples.invalid,
                        }),
                        _ => Value::Null,
                    };
                    map.insert(column.clone(), value);
                }
                Value::Object(map)
            }
        })
        .collect::<Vec<_>>();

    let columns = if select_all && !rows.is_empty() {
        rows[0]
            .as_object()
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default()
    } else {
        columns
    };

    Ok(SqlQueryResult { columns, rows })
}

fn execute_describe_query(table: String) -> Result<SqlQueryResult, String> {
    let rules = match table.to_ascii_lowercase().as_str() {
        "validation_rules" => validation_rules(),
        _ => return Err(format!("Unknown table: {table}")),
    };

    let Some(sample) = rules.first() else {
        return Ok(SqlQueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
        });
    };

    let columns = vec![
        "Field".to_string(),
        "Type".to_string(),
        "Description".to_string(),
    ];

    let mut rows = Vec::new();
    let value = serde_json::to_value(sample).map_err(|e| e.to_string())?;
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            let type_str = match val {
                Value::Array(_) => "object",
                Value::Null => "object",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Object(_) => "object",
            };
            rows.push(json!({
                "Field": key,
                "Type": type_str,
                "Description": format!("Field {key} of type {type_str}"),
            }));
        }
    }

    Ok(SqlQueryResult { columns, rows })
}

fn evaluate_where_clause(item: &ValidationRule, clause: &Condition) -> bool {
    match clause {
        Condition::And(conditions) => conditions.iter().all(|c| evaluate_where_clause(item, c)),
        Condition::Or(conditions) => conditions.iter().any(|c| evaluate_where_clause(item, c)),
        Condition::Comparison {
            column,
            operator,
            value,
        } => evaluate_condition(item, column, *operator, value),
    }
}

fn evaluate_condition(
    item: &ValidationRule,
    column: &str,
    operator: Operator,
    value: &Value,
) -> bool {
    let item_value = match column {
        "id" => Value::String(item.id.clone()),
        "name" => Value::String(item.name.clone()),
        "description" => Value::String(item.description.clone()),
        "category" => Value::String(item.category.clone()),
        "fileTypes" | "file_types" => json!(
            item.file_types.iter().map(|ft| ft.as_str()).collect::<Vec<_>>()
        ),
        _ => Value::Null,
    };

    match operator {
        Operator::Eq => item_value == *value,
        Operator::Neq => item_value != *value,
        Operator::Gt => compare_numbers(&item_value, value, |a, b| a > b),
        Operator::Lt => compare_numbers(&item_value, value, |a, b| a < b),
        Operator::Gte => compare_numbers(&item_value, value, |a, b| a >= b),
        Operator::Lte => compare_numbers(&item_value, value, |a, b| a <= b),
        Operator::Like => {
            if let Some(text) = item_value.as_str() {
                let pattern = value
                    .as_str()
                    .unwrap_or("")
                    .replace('%', ".*")
                    .replace('_', ".");
                Regex::new(&format!("(?i)^{pattern}$"))
                    .map(|re| re.is_match(text))
                    .unwrap_or(false)
            } else {
                false
            }
        }
        Operator::In => {
            if let Some(arr) = value.as_array() {
                arr.iter().any(|v| v == &item_value)
            } else {
                false
            }
        }
    }
}

fn compare_numbers<F>(left: &Value, right: &Value, cmp: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    let Some(l) = left.as_f64() else { return false };
    let Some(r) = right.as_f64() else { return false };
    cmp(l, r)
}

fn sort_data(data: &mut [ValidationRule], order_by: &[OrderBy]) {
    data.sort_by(|a, b| {
        for order in order_by {
            let a_val = field_to_value(a, &order.column);
            let b_val = field_to_value(b, &order.column);
            let ord = match (a_val, b_val) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
            if ord != std::cmp::Ordering::Equal {
                return match order.direction {
                    SortDirection::Asc => ord,
                    SortDirection::Desc => ord.reverse(),
                };
            }
        }
        std::cmp::Ordering::Equal
    });
}

fn field_to_value(rule: &ValidationRule, field: &str) -> Option<String> {
    match field {
        "id" => Some(rule.id.clone()),
        "name" => Some(rule.name.clone()),
        "description" => Some(rule.description.clone()),
        "category" => Some(rule.category.clone()),
        "fileTypes" | "file_types" => Some(
            rule.file_types
                .iter()
                .map(|ft| ft.as_str())
                .collect::<Vec<_>>()
                .join(","),
        ),
        _ => None,
    }
}
