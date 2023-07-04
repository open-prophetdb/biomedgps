//! A SQL builder for building SQL queries.

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    ArrayString(Vec<String>),
    ArrayInt(Vec<i32>),
    ArrayFloat(Vec<f64>),
    ArrayBool(Vec<bool>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueryItem {
    pub field: String,
    pub value: Value,
    pub operator: String, // =, !=, like, not like, in, not in
}

impl QueryItem {
    pub fn new(field: String, value: Value, operator: String) -> Self {
        let allowed_operators = vec!["=", "!=", "like", "not like", "in", "not in"];
        if !allowed_operators.contains(&operator.as_str()) {
            panic!("Invalid operator: {}", operator);
        }

        match value {
            Value::Int(_) => {
                if !vec!["=", "!="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Float(_) => {
                if !vec!["=", "!="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::String(_) => {
                if !vec!["=", "!=", "like", "not like"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Bool(_) => {
                if !vec!["=", "!="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Null => {
                if !vec!["=", "!="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayString(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayInt(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayFloat(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::ArrayBool(_) => {
                if !vec!["in", "not in"].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
        }

        Self {
            field,
            value,
            operator,
        }
    }

    pub fn default() -> Self {
        QueryItem::new(
            "1".to_string(),
            Value::String("1".to_string()),
            "=".to_string(),
        )
    }

    pub fn format(&self) -> String {
        match &self.value {
            Value::Int(v) => format!("{} {} {}", self.field, self.operator, v),
            Value::Float(v) => format!("{} {} {}", self.field, self.operator, v),
            Value::String(v) => format!("{} {} '{}'", self.field, self.operator, v),
            Value::Bool(v) => format!("{} {} {}", self.field, self.operator, v),
            Value::Null => format!("{} {} NULL", self.field, self.operator),
            Value::ArrayString(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("'{}'", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
            Value::ArrayInt(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("{}", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
            Value::ArrayFloat(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("{}", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
            Value::ArrayBool(v) => {
                let mut values = vec![];
                for item in v {
                    values.push(format!("{}", item));
                }
                format!("{} {} ({})", self.field, self.operator, values.join(","))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComposeQueryItem {
    /// and, or
    pub operator: String,
    /// QueryItem or ComposeQuery
    pub items: Vec<ComposeQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ComposeQuery {
    QueryItem(QueryItem),
    ComposeQueryItem(ComposeQueryItem),
}

impl ComposeQueryItem {
    pub fn new(operator: &str) -> Self {
        Self {
            operator: operator.to_string(),
            items: vec![],
        }
    }

    // Why ComposeQuery here?
    // Because we can have nested ComposeQueryItem, it maybe a QueryItem or ComposeQueryItem
    pub fn add_item(&mut self, item: ComposeQuery) {
        self.items.push(item);
    }

    pub fn default() -> Self {
        let mut default_query = ComposeQueryItem::new("and");
        default_query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "1".to_string(),
            Value::Int(1),
            "=".to_string(),
        )));

        default_query
    }

    pub fn format(&self) -> String {
        let mut query = String::new();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                query.push_str(&format!(" {} ", self.operator));
            }

            match item {
                ComposeQuery::QueryItem(item) => {
                    query.push_str(&item.format());
                }
                ComposeQuery::ComposeQueryItem(item) => {
                    query.push_str(&format!("({})", item.format()));
                }
            }
        }
        query
    }
}

// Test code
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_query() {
        let mut query = ComposeQueryItem::new("and");
        query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "id".to_string(),
            Value::Int(1),
            "=".to_string(),
        )));
        query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "name".to_string(),
            Value::String("test".to_string()),
            "like".to_string(),
        )));

        let mut compose_query = ComposeQueryItem::new("or");
        compose_query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "id".to_string(),
            Value::Int(2),
            "=".to_string(),
        )));
        compose_query.add_item(ComposeQuery::QueryItem(QueryItem::new(
            "name".to_string(),
            Value::String("test2".to_string()),
            "like".to_string(),
        )));

        query.add_item(ComposeQuery::ComposeQueryItem(compose_query));

        assert_eq!(
            query.format(),
            "id = 1 and name like 'test' and (id = 2 or name like 'test2')"
        );
    }
}
