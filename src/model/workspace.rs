//! The workspace model is used to store the information of workspaces which are created by the user. Also, the workspace is the container of the workflow, task etc.

use crate::model::core::CheckData;
use crate::query_builder::sql_builder::ComposeQuery;
use anyhow::Ok as AnyOk;
use chrono::{serde::ts_seconds, DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::error::Error;
use std::path::PathBuf;
use uuid::Uuid;
use validator::Validate;

const DEFAULT_LENGTH_1: usize = 1;
const DEFAULT_LENGTH_16: usize = 16;
const DEFAULT_LENGTH_32: usize = 32;
const DEFAULT_LENGTH_36: usize = 36;
const DEFAULT_LENGTH_64: usize = 64;
const DEFAULT_LENGTH_255: usize = 255;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Workspace {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    #[oai(skip)]
    pub id: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_64",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 64."
    ))]
    workspace_name: String,

    #[oai(skip_serializing_if_is_none)]
    description: Option<String>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    created_time: DateTime<Utc>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    updated_time: DateTime<Utc>,

    #[serde(skip_deserializing)]
    #[oai(read_only)]
    archived_time: Option<DateTime<Utc>>,

    #[oai(skip_serializing_if_is_none)]
    payload: Option<JsonValue>,

    owner: String,
    groups: Vec<String>,
}

impl Workspace {
    pub fn update_owner(&mut self, owner: String) -> &Self {
        self.owner = owner;
        return self;
    }

    pub fn update_groups(&mut self, groups: Vec<String>) -> &Self {
        self.groups = groups;
        return self;
    }

    pub async fn get_records_by_owner(
        pool: &sqlx::PgPool,
        owner: &str,
        query: Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<Vec<Workspace>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let pagination_str = if page.is_none() && page_size.is_none() {
            "LIMIT 10 OFFSET 0".to_string()
        } else {
            let page = match page {
                Some(page) => page,
                None => 1,
            };

            let page_size = match page_size {
                Some(page_size) => page_size,
                None => 10,
            };

            let limit = page_size;
            let offset = (page - 1) * page_size;

            format!("LIMIT {} OFFSET {}", limit, offset)
        };

        let where_str = format!("owner = $1 AND ({})", query_str);

        let sql_str = format!(
            "SELECT id, workspace_name, description, created_time, updated_time, archived_time, payload, owner, groups FROM biomedgps_workspace WHERE {} {} {}",
            where_str, order_by_str, pagination_str
        );

        let records = sqlx::query_as::<_, Workspace>(sql_str.as_str())
            .bind(owner)
            .fetch_all(pool)
            .await?;

        AnyOk(records)
    }

    pub async fn insert_record(
        pool: &sqlx::PgPool,
        name: &str,
        description: Option<&str>,
        owner: &str,
        groups: Option<Vec<&str>>,
    ) -> Result<Workspace, anyhow::Error> {
        let groups = match groups {
            Some(groups) => groups,
            None => vec![],
        };

        let sql_str = "
        INSERT INTO biomedgps_workspace (workspace_name, description, owner, groups) 
        VALUES ($1, $2, $3, $4)
        RETURNING *"; // Add RETURNING to get the inserted row

        let workspace = sqlx::query_as::<_, Workspace>(sql_str)
            .bind(name)
            .bind(description)
            .bind(owner)
            .bind(groups) // Ensure PostgreSQL can handle Option<Vec<&str>> appropriately
            .fetch_one(pool)
            .await?;

        AnyOk(workspace)
    }
}

impl CheckData for Workspace {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Workspace>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        return vec![];
    }

    fn fields() -> Vec<String> {
        vec![
            "workspace_name".to_string(),
            "description".to_string(),
            "payload".to_string(),
            "owner".to_string(),
            "groups".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Workflow {
    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
    id: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of name should be between 1 and 255."
    ))]
    name: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of version should be between 1 and 255."
    ))]
    version: String,

    description: Option<String>,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 255."
    ))]
    category: String,

    home: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 255."
    ))]
    source: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 255."
    ))]
    short_name: String,

    icons: Option<JsonValue>,

    #[validate(length(
        max = "DEFAULT_LENGTH_64",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 64."
    ))]
    author: String,

    maintainers: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    readme: Option<String>,
}

impl CheckData for Workflow {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Workflow>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["id".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            "id".to_string(),
            "name".to_string(),
            "version".to_string(),
            "description".to_string(),
            "category".to_string(),
            "home".to_string(),
            "source".to_string(),
            "short_name".to_string(),
            "icons".to_string(),
            "author".to_string(),
            "maintainers".to_string(),
            "tags".to_string(),
            "readme".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct WorkflowSchema {
    schema: JsonValue,
}

impl Workflow {
    pub async fn get_workflow_schema(
        pool: &sqlx::PgPool,
        id: &str,
        workflow_dir: &PathBuf,
    ) -> Result<WorkflowSchema, anyhow::Error> {
        let sql_str = format!("SELECT payload FROM biomedgps_workflow WHERE id = $1");
        let workflow = sqlx::query_as::<_, Workflow>(sql_str.as_str())
            .bind(id)
            .fetch_one(pool)
            .await?;

        let workflow_name = format!("{}-{}", workflow.short_name, workflow.version);
        let workflow_path = workflow_dir.join(workflow_name);
        let schema_path = workflow_path.join("schema.json");
        let schema = std::fs::read_to_string(schema_path)?;
        let schema: JsonValue = serde_json::from_str(&schema)?;

        AnyOk(WorkflowSchema { schema })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Task {
    // Ignore this field when deserialize from json
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    #[oai(skip)]
    id: i64,

    #[validate(length(
        max = "DEFAULT_LENGTH_36",
        min = "DEFAULT_LENGTH_36",
        message = "The length of id should be 36."
    ))]
    workspace_id: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_36",
        min = "DEFAULT_LENGTH_36",
        message = "The length of id should be 36."
    ))]
    workflow_id: String,

    #[serde(skip_deserializing)]
    #[oai(read_only)]
    #[oai(skip)]
    task_id: String,

    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
    task_name: String,

    description: Option<String>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    submitted_time: DateTime<Utc>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    started_time: DateTime<Utc>,

    #[serde(skip_deserializing)]
    #[serde(with = "ts_seconds")]
    #[oai(read_only)]
    finished_time: DateTime<Utc>,

    task_params: JsonValue,

    #[oai(skip_serializing_if_is_none)]
    labels: Option<Vec<String>>,

    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    #[oai(skip)]
    status: Option<String>,

    #[serde(skip_deserializing)]
    #[oai(read_only)]
    #[oai(skip)]
    log_message: Option<String>,

    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
    owner: String,

    #[oai(skip_serializing_if_is_none)]
    groups: Option<Vec<String>>,
}

impl CheckData for Task {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Task>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["workspace_id".to_string(), "task_id".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            "workspace_id".to_string(),
            "workflow_id".to_string(),
            "task_id".to_string(),
            "task_name".to_string(),
            "description".to_string(),
            "submitted_time".to_string(),
            "started_time".to_string(),
            "finished_time".to_string(),
            "task_params".to_string(),
            "labels".to_string(),
            "status".to_string(),
            "owner".to_string(),
            "log_message".to_string(),
            "groups".to_string(),
        ]
    }
}

impl Task {
    pub fn update_owner(&mut self, owner: String) -> &Self {
        self.owner = owner;
        return self;
    }

    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<Task, anyhow::Error> {
        let sql_str = "INSERT INTO biomedgps_task (workspace_id, workflow_id, task_id, task_name, description, task_params, labels, owner, groups) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *";

        let task_id = Uuid::new_v4().to_string();

        let task = sqlx::query_as::<_, Task>(sql_str)
            .bind(&self.workspace_id)
            .bind(&self.workflow_id)
            .bind(task_id)
            .bind(&self.task_name)
            .bind(&self.description)
            .bind(&self.task_params)
            .bind(&self.labels)
            .bind(&self.owner)
            .bind(&self.groups)
            .fetch_one(pool)
            .await?;

        AnyOk(task)
    }

    pub async fn get_records_by_id(
        pool: &sqlx::PgPool,
        task_id: &str,
        owner: &str,
    ) -> Result<Task, anyhow::Error> {
        let sql_str = "SELECT * FROM biomedgps_task WHERE task_id = $1 AND owner = $2";
        let task = sqlx::query_as::<_, Task>(sql_str)
            .bind(task_id)
            .bind(owner)
            .fetch_one(pool)
            .await?;

        AnyOk(task)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Notification {
    id: String,
    title: String,
    description: Option<String>,
    notification_type: String,
    created_time: DateTime<Utc>,
    status: String,
    owner: String,
}

impl CheckData for Notification {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<Notification>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec![]
    }

    fn fields() -> Vec<String> {
        vec![
            "title".to_string(),
            "description".to_string(),
            "notification_type".to_string(),
            "created_time".to_string(),
            "status".to_string(),
            "owner".to_string(),
        ]
    }
}

impl Notification {
    pub async fn get_records_by_owner(
        pool: &sqlx::PgPool,
        owner: &str,
        query: Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<Vec<Notification>, anyhow::Error> {
        let columns = <Notification as CheckData>::fields().join(",");
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let where_str = format!("owner = $1 AND ({})", query_str);

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let pagination_str = if page.is_none() && page_size.is_none() {
            "LIMIT 10 OFFSET 0".to_string()
        } else {
            let page = match page {
                Some(page) => page,
                None => 1,
            };

            let page_size = match page_size {
                Some(page_size) => page_size,
                None => 10,
            };

            format!("LIMIT {} OFFSET {}", page_size, (page - 1) * page_size)
        };

        let sql_str = format!(
            "SELECT id, {columns} FROM biomedgps_notification WHERE {} {} {}",
            where_str, order_by_str, pagination_str
        );

        let records = sqlx::query_as::<_, Notification>(sql_str.as_str())
            .bind(owner)
            .fetch_all(pool)
            .await?;

        AnyOk(records)
    }
}
