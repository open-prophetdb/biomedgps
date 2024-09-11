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
    pub operator: String, // =, !=, like, not like, ilike, in, not in
}

impl QueryItem {
    pub fn new(field: String, value: Value, operator: String) -> Self {
        let allowed_operators = vec![
            "=", "!=", "like", "not like", "ilike", "in", "not in", "<>", "<", ">", "<=", ">=",
            "is", "is not",
        ];
        if !allowed_operators.contains(&operator.as_str()) {
            panic!("Invalid operator: {}", operator);
        }

        match value {
            Value::Int(_) => {
                if !vec!["=", "!=", ">", "<", "<=", ">="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Float(_) => {
                if !vec!["=", "!=", ">", "<", "<=", ">="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::String(_) => {
                if !vec!["=", "!=", "like", "not like", "ilike", "<>"].contains(&operator.as_str())
                {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Bool(_) => {
                if !vec!["=", "!="].contains(&operator.as_str()) {
                    panic!("Invalid operator: {}", operator);
                }
            }
            Value::Null => {
                if !vec!["is", "is not"].contains(&operator.as_str()) {
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

    pub fn get_field(&self) -> &str {
        &self.field
    }

    pub fn get_field_value_pair(&self) -> Option<(String, String)> {
        // Only return a string value
        match &self.value {
            Value::String(v) => Some((self.field.clone(), v.clone())),
            _ => None,
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

impl ComposeQuery {
    pub fn to_string(&self) -> String {
        let mut query_str = match self {
            ComposeQuery::QueryItem(item) => item.format(),
            ComposeQuery::ComposeQueryItem(item) => item.format(),
        };

        query_str
    }

    pub fn from_str(query_str: &str) -> Result<Option<Self>, serde_json::Error> {
        let query = if query_str == "" {
            None
        } else {
            Some(serde_json::from_str(&query_str)?)
        };

        Ok(query)
    }
}

impl ComposeQueryItem {
    pub fn new(operator: &str) -> Self {
        Self {
            operator: operator.to_string(),
            items: vec![],
        }
    }

    pub fn get_fields(&self, fields: &mut Vec<String>) {
        for item in &self.items {
            match item {
                ComposeQuery::QueryItem(query_item) => {
                    // Check if the field is not already in the Vec and add it
                    if !fields.contains(&query_item.field) {
                        fields.push(query_item.field.clone());
                    }
                }
                ComposeQuery::ComposeQueryItem(compose_query_item) => {
                    // Recursively traverse nested ComposeQueryItem
                    compose_query_item.get_fields(fields);
                }
            }
        }
    }

    pub fn get_field_value_pairs(&self, pairs: &mut Vec<(String, String)>) {
        for item in &self.items {
            match item {
                ComposeQuery::QueryItem(query_item) => {
                    // Check if the field is not already in the Vec and add it
                    if let Some(pair) = query_item.get_field_value_pair() {
                        if !pairs.contains(&pair) {
                            pairs.push(pair);
                        }
                    }
                }
                ComposeQuery::ComposeQueryItem(compose_query_item) => {
                    // Recursively traverse nested ComposeQueryItem
                    compose_query_item.get_field_value_pairs(pairs);
                }
            }
        }
    }

    // Why ComposeQuery here?
    // Because we can have nested ComposeQueryItem, it maybe a QueryItem or ComposeQueryItem
    pub fn add_item(&mut self, item: ComposeQuery) -> &mut Self {
        self.items.push(item);
        self
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

pub fn get_all_fields(query: &ComposeQuery) -> Vec<String> {
    match query {
        ComposeQuery::QueryItem(query_item) => {
            let mut fields = Vec::new();
            fields.push(query_item.get_field().to_string());
            return fields;
        }
        ComposeQuery::ComposeQueryItem(query) => {
            let mut fields = Vec::new();
            query.get_fields(&mut fields);
            return fields;
        }
    }
}

pub fn get_all_field_pairs(query: &ComposeQuery) -> Vec<(String, String)> {
    match query {
        ComposeQuery::QueryItem(query_item) => {
            let mut pairs = Vec::new();
            if let Some(pair) = query_item.get_field_value_pair() {
                pairs.push(pair);
            }
            return pairs;
        }
        ComposeQuery::ComposeQueryItem(query) => {
            let mut pairs = Vec::new();
            query.get_field_value_pairs(&mut pairs);
            return pairs;
        }
    }
}

pub fn make_order_clause(fields: Vec<String>) -> String {
    let mut order_by = String::new();
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            order_by.push_str(", ");
        }
        order_by.push_str(field);
    }
    order_by
}

pub fn make_order_clause_by_pairs(pairs: Vec<(String, String)>, topk: usize) -> String {
    let mut topk_pairs = Vec::new();
    if topk != 0 {
        let k = if pairs.len() < topk {
            pairs.len()
        } else {
            topk
        };
        topk_pairs = pairs[0..k].to_vec();
    } else {
        topk_pairs = pairs;
    }

    let mut order_by = String::new();
    for (i, pair) in topk_pairs.iter().enumerate() {
        if i > 0 {
            order_by.push_str(", ");
        }

        // Trim all special characters in the head and tail of the string
        let patterns: &[_] = &[
            '~', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '+', '=', '{', '}', '[',
            ']', '|', '\\', ':', ';', '"', '\'', '<', '>', ',', '.', '?', '/', ' ',
        ];
        let cleaned_str = pair.1.trim_matches(patterns);
        order_by.push_str(&format!("similarity({}, '{}') DESC", pair.0, cleaned_str));
    }
    order_by
}

// Test code
#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use log::LevelFilter;

    #[test]
    fn test_compose_query() {
        let _ = init_logger("sql-builder-test", LevelFilter::Debug);
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

        let mut fields = Vec::new();
        query.get_fields(&mut fields);
        debug!("fields: {:?}", fields);
        assert_eq!(2, fields.len());

        let mut pairs = Vec::new();
        query.get_field_value_pairs(&mut pairs);
        debug!("pairs: {:?}", pairs);
        assert_eq!(2, pairs.len());
    }
}
