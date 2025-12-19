use regex::Regex;
use serde_json::{json, Map, Value};

use crate::tools::types::{FileType, ValidationRule};
use crate::tools::validator::validation_rules;

pub fn query_validation_rules(query: &str, file_type: Option<FileType>) -> Vec<ValidationRule> {
    let normalized = query.to_lowercase().trim().to_string();
    let rules = validation_rules();

    if normalized.is_empty() {
        return match file_type {
            Some(target) => rules
                .into_iter()
                .filter(|rule| rule.file_types.contains(&target))
                .collect(),
            None => rules,
        };
    }

    rules
        .into_iter()
        .filter(|rule| {
            if let Some(target) = file_type {
                if !rule.file_types.contains(&target) {
                    return false;
                }
            }

            rule.id.to_lowercase().contains(&normalized)
                || rule.name.to_lowercase().contains(&normalized)
                || rule.description.to_lowercase().contains(&normalized)
                || rule.category.to_lowercase().contains(&normalized)
        })
        .collect()
}

pub fn get_validation_rule_by_id(id: &str) -> Option<ValidationRule> {
    validation_rules().into_iter().find(|rule| rule.id == id)
}

pub fn get_validation_rules_by_category(category: &str) -> Vec<ValidationRule> {
    validation_rules()
        .into_iter()
        .filter(|rule| rule.category == category)
        .collect()
}

pub fn get_validation_rules_by_file_type(file_type: FileType) -> Vec<ValidationRule> {
    validation_rules()
        .into_iter()
        .filter(|rule| rule.file_types.contains(&file_type))
        .collect()
}

pub fn get_categories() -> Vec<String> {
    let mut categories = validation_rules()
        .into_iter()
        .map(|rule| rule.category)
        .collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories
}

pub fn get_validation_rules_with_examples(
    query: Option<&str>,
    file_type: Option<FileType>,
) -> Vec<ValidationRule> {
    match (query, file_type) {
        (Some(q), ft) => query_validation_rules(q, ft),
        (None, Some(ft)) => get_validation_rules_by_file_type(ft),
        _ => validation_rules(),
    }
}

pub fn execute_sql_query(sql_query: &str) -> Result<Vec<Value>, String> {
    let re = Regex::new(r"(?i)^SELECT\s+(.*?)\s+FROM\s+(.*?)(?:\s+WHERE\s+(.*?))?(?:\s+ORDER\s+BY\s+(.*?))?(?:\s+LIMIT\s+(\d+))?$")
        .map_err(|err| format!("Failed to compile query regex: {err}"))?;

    let caps = re
        .captures(sql_query.trim())
        .ok_or_else(|| "Invalid SQL query format".to_string())?;

    let select_clause = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
    let from_clause = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
    let where_clause = caps.get(3).map(|m| m.as_str());
    let order_by_clause = caps.get(4).map(|m| m.as_str());
    let limit_clause = caps
        .get(5)
        .and_then(|m| m.as_str().parse::<usize>().ok());

    if from_clause.trim().eq_ignore_ascii_case("validation_rules") == false {
        return Err("Only validation_rules table is supported".to_string());
    }

    let select_all = select_clause.trim() == "*";
    let selected_fields: Vec<String> = if select_all {
        vec![
            "id".to_string(),
            "description".to_string(),
            "type".to_string(),
            "category".to_string(),
            "severity".to_string(),
            "documentation".to_string(),
        ]
    } else {
        select_clause
            .split(',')
            .map(|f| f.trim().to_string())
            .collect()
    };

    let mut filtered_rules = validation_rules();

    if let Some(where_clause) = where_clause {
        filtered_rules = filtered_rules
            .into_iter()
            .filter(|rule| {
                where_clause
                    .split("AND")
                    .filter(|c| !c.trim().is_empty())
                    .all(|cond| matches_condition(rule, cond.trim()))
            })
            .collect();
    }

    if let Some(order_clause) = order_by_clause {
        let mut parts = order_clause.trim().split_whitespace();
        if let Some(field) = parts.next() {
            let desc = parts
                .next()
                .map(|v| v.eq_ignore_ascii_case("DESC"))
                .unwrap_or(false);
            filtered_rules.sort_by(|a, b| {
                let a_val = field_value(a, field);
                let b_val = field_value(b, field);
                match (a_val, b_val) {
                    (Some(av), Some(bv)) => {
                        if desc { bv.cmp(&av) } else { av.cmp(&bv) }
                    }
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }
    }

    if let Some(limit) = limit_clause {
        filtered_rules.truncate(limit);
    }

    let rows = filtered_rules
        .into_iter()
        .map(|rule| project_fields(&rule, &selected_fields, select_all))
        .collect();

    Ok(rows)
}

fn matches_condition(rule: &ValidationRule, condition: &str) -> bool {
    let condition = condition.trim();
    if condition.is_empty() {
        return true;
    }

    // Use standard string literals with properly escaped backslashes and quotes
    let equality = Regex::new("(?i)(\\w+)\\s*=\\s*['\"]?(.*?)['\"]?$").unwrap();
    let like = Regex::new("(?i)(\\w+)\\s+LIKE\\s+['\"]%(.*?)%['\"]").unwrap();

    if let Some(caps) = equality.captures(condition) {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let value = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_lowercase();
        return field_value(rule, field)
            .map(|v| v.to_lowercase() == value)
            .unwrap_or(false);
    }

    if let Some(caps) = like.captures(condition) {
        let field = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let value = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_lowercase();
        return field_value(rule, field)
            .map(|v| v.to_lowercase().contains(&value))
            .unwrap_or(false);
    }

    true
}

fn field_value(rule: &ValidationRule, field: &str) -> Option<String> {
    match field {
        "id" => Some(rule.id.clone()),
        "name" => Some(rule.name.clone()),
        "description" => Some(rule.description.clone()),
        "category" => Some(rule.category.clone()),
        "fileTypes" | "file_types" => Some(
            rule.file_types
                .iter()
                .map(|ft| ft.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
        _ => None,
    }
}

fn project_fields(rule: &ValidationRule, fields: &[String], select_all: bool) -> Value {
    let mut map = Map::new();
    if select_all {
        map.insert("id".to_string(), Value::String(rule.id.clone()));
        map.insert("name".to_string(), Value::String(rule.name.clone()));
        map.insert(
            "description".to_string(),
            Value::String(rule.description.clone()),
        );
        map.insert("category".to_string(), Value::String(rule.category.clone()));
        map.insert(
            "fileTypes".to_string(),
            json!(rule.file_types.iter().map(|ft| ft.as_str()).collect::<Vec<_>>()),
        );
        map.insert(
            "examples".to_string(),
            json!({
                "valid": rule.examples.valid,
                "invalid": rule.examples.invalid,
            }),
        );
    } else {
        for field in fields {
            let value = match field.as_str() {
                "id" => Some(Value::String(rule.id.clone())),
                "name" => Some(Value::String(rule.name.clone())),
                "description" => Some(Value::String(rule.description.clone())),
                "category" => Some(Value::String(rule.category.clone())),
                "fileTypes" | "file_types" => Some(json!(
                    rule.file_types.iter().map(|ft| ft.as_str()).collect::<Vec<_>>()
                )),
                "examples" => Some(json!({
                    "valid": rule.examples.valid,
                    "invalid": rule.examples.invalid,
                })),
                _ => None,
            };

            map.insert(
                field.clone(),
                value.unwrap_or_else(|| Value::String(String::new())),
            );
        }
    }

    Value::Object(map)
}
