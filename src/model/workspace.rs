//! The workspace model is used to store the information of workspaces which are created by the user. Also, the workspace is the container of the workflow, task etc.

use crate::model::core::CheckData;
use crate::query_builder::sql_builder::ComposeQuery;
use anyhow::Ok as AnyOk;
use chrono::{serde::ts_seconds, DateTime, Utc};
use log::warn;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::Row;
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
    readme: String,
    schema: JsonValue,
}

impl Workflow {
    pub async fn get_workflow_schema(
        pool: &sqlx::PgPool,
        id: &str,
        workflow_dir: &PathBuf,
    ) -> Result<WorkflowSchema, anyhow::Error> {
        let sql_str = format!("SELECT * FROM biomedgps_workflow WHERE id = $1");
        let workflow = sqlx::query_as::<_, Workflow>(sql_str.as_str())
            .bind(id)
            .fetch_one(pool)
            .await?;

        let workflow_name = format!("{}-{}", workflow.short_name, workflow.version);
        let workflow_path = workflow_dir.join(workflow_name);
        let schema_path = workflow_path.join("schema.json");
        let schema = std::fs::read_to_string(schema_path)?;
        let schema: WorkflowSchema = serde_json::from_str(&schema)?;

        AnyOk(schema)
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
    #[oai(read_only)]
    started_time: Option<DateTime<Utc>>,

    #[serde(skip_deserializing)]
    #[oai(read_only)]
    finished_time: Option<DateTime<Utc>>,

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
    status: Option<String>,

    #[serde(skip_deserializing)]
    #[oai(read_only)]
    results: Option<JsonValue>, // {"files": [{"filelink": "...", "filetype": "tsv"}, {"filelink": "...", "filetype": "csv"}], "charts": [{"filelink": "...", "filetype": "plotly"}, {"filelink": "...", "filetype": "png"}]}

    #[serde(skip_deserializing)]
    #[oai(read_only)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow)]
pub struct ExpandedTask {
    task: Task,
    workflow: Workflow,
}

impl ExpandedTask {
    pub async fn get_records_by_id(
        pool: &sqlx::PgPool,
        task_id: &str,
        owner: &str,
        task_root_dir: &PathBuf,
        expand_results: bool,
    ) -> Result<ExpandedTask, anyhow::Error> {
        // Updated query to explicitly select columns from both tables
        let sql_str = "
            SELECT 
                biomedgps_task.id,
                biomedgps_task.workspace_id,
                biomedgps_task.workflow_id,
                biomedgps_task.task_id,
                biomedgps_task.task_name,
                biomedgps_task.description,
                biomedgps_task.submitted_time,
                biomedgps_task.started_time,
                biomedgps_task.finished_time,
                biomedgps_task.task_params,
                biomedgps_task.labels,
                biomedgps_task.status,
                biomedgps_task.owner,
                biomedgps_task.groups,
                biomedgps_task.results,
                biomedgps_task.log_message,
                biomedgps_workflow.id AS workflow_id,
                biomedgps_workflow.name AS workflow_name,
                biomedgps_workflow.version,
                biomedgps_workflow.description AS workflow_description,
                biomedgps_workflow.category,
                biomedgps_workflow.home,
                biomedgps_workflow.source,
                biomedgps_workflow.short_name,
                biomedgps_workflow.icons,
                biomedgps_workflow.author,
                biomedgps_workflow.maintainers,
                biomedgps_workflow.tags,
                biomedgps_workflow.readme
            FROM biomedgps_task 
            JOIN biomedgps_workflow 
            ON biomedgps_task.workflow_id = biomedgps_workflow.id
            WHERE biomedgps_task.task_id = $1 
            AND biomedgps_task.owner = $2
        ";

        let rows = match sqlx::query(sql_str)
            .bind(task_id)
            .bind(owner)
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows,
            Err(sqlx::Error::RowNotFound) => {
                return Err(anyhow::anyhow!("No task or workflow found"))
            }
            Err(e) => return Err(anyhow::anyhow!("Failed to get expanded task: {}", e)),
        };

        if rows.len() == 1 {
            let row = &rows[0];

            let mut task = Task {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                workflow_id: row.get("workflow_id"),
                task_id: row.get("task_id"),
                task_name: row.get("task_name"),
                description: row.get("description"),
                submitted_time: row.get("submitted_time"),
                started_time: row.get("started_time"),
                finished_time: row.get("finished_time"),
                task_params: row.get("task_params"),
                labels: row.get("labels"),
                status: row.get("status"),
                results: row.get("results"),
                log_message: row.get("log_message"),
                owner: row.get("owner"),
                groups: row.get("groups"),
            };

            let workflow = Workflow {
                id: row.get("workflow_id"),
                name: row.get("workflow_name"),
                version: row.get("version"),
                description: row.get("workflow_description"),
                category: row.get("category"),
                home: row.get("home"),
                source: row.get("source"),
                short_name: row.get("short_name"),
                icons: row.get("icons"),
                author: row.get("author"),
                maintainers: row.get("maintainers"),
                tags: row.get("tags"),
                readme: row.get("readme"),
            };

            // The status of the task contains Succeeded, Submitted, Failed, Running, Pending, etc in Cromwell server.
            if task.status.is_some()
                && task.status == Some("Succeeded".to_string())
                && expand_results
            {
                let workflow_short_name = &workflow.short_name;
                // We expect all workflows have a task named "<workflow.short_name>", and the output directory is call-<workflow.short_name> in the workflow directory. Each workflow only has one such task. The task name is same as the workflow short name. such as:
                // <ROOT_DIR>/<workflow.short_name>/<TASK_ID>/
                //   |- call-<workflow.short_name>/
                //   |      |- out.txt
                //   |      |- output.json
                //
                // NOTE:
                // 1. The execution directory might exist or not, if not, the output files will be in the same directory as the task directory.
                // 2. We also expect all workflows can output a file named `metadata.json`, which contains the information of all output files. such as:
                //    {
                //      "files": [
                //        {"filelink": "out.txt", "filetype": "tsv", ...other fields...},
                //        {"filelink": "output.json", "filetype": "json", ...other fields...}
                //      ],
                //      "charts": [
                //        {"filelink": "chart.png", "filetype": "png", ...other fields...}
                //      ]
                //    }
                // 3. We also expect all output files can be copied to the directory of the output task. such as: <ROOT_DIR>/<workflow.short_name>/<TASK_ID>/call-<workflow.short_name>/out.txt
                let output_task_dir_name = format!("call-{}", workflow_short_name);
                let task_id = &task.task_id;
                let output_task_dir = task_root_dir
                    .join(workflow_short_name)
                    .join(task_id)
                    .join(output_task_dir_name);
                let output_metadata_file = output_task_dir.join("metadata.json");

                if output_metadata_file.exists() {
                    let metadata = std::fs::read_to_string(output_metadata_file)?;
                    let metadata: JsonValue = serde_json::from_str(&metadata)?;

                    task.update_results(Some(metadata));
                } else {
                    let msg = format!("Output metadata file not found: {}, it might not a valid workflow or the task is not finished.", output_metadata_file.display());
                    warn!("{}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }

            AnyOk(ExpandedTask { task, workflow })
        } else {
            Err(anyhow::anyhow!("No task or workflow found"))
        }
    }

    pub async fn get_log(
        pool: &sqlx::PgPool,
        owner: &str,
        task_root_dir: &PathBuf,
        task_id: &str,
    ) -> Result<String, anyhow::Error> {
        let expanded_task = ExpandedTask::get_records_by_id(pool, task_id, owner, task_root_dir, false).await?;
        
        let workflow_short_name = &expanded_task.workflow.short_name;
        let task_dir = task_root_dir
            .join(workflow_short_name)
            .join(task_id)
            .join(format!("call-{}", workflow_short_name));

        // TODO: We assume the log file is named "stderr" and "stdout" in the task directory. In the current implementation, we don't support files on the cloud storage.
        let stderr_log_file = task_dir.join("stderr");
        let stdout_log_file = task_dir.join("stdout");

        let mut log_content = String::new();
        if stderr_log_file.exists() {
            log_content += &std::fs::read_to_string(stderr_log_file)?;
        }
        
        if stdout_log_file.exists() {
            log_content += &std::fs::read_to_string(stdout_log_file)?;
        }

        if log_content.is_empty() {
            return Err(anyhow::anyhow!("Log file not found in the task directory: {}", task_id));
        }

        AnyOk(log_content)
    }

    pub async fn get_file(
        pool: &sqlx::PgPool,
        owner: &str,
        task_root_dir: &PathBuf,
        task_id: &str,
        file_name: &str,
    ) -> Result<PathBuf, anyhow::Error> {
        let expanded_task =
            ExpandedTask::get_records_by_id(pool, task_id, owner, task_root_dir, false).await?;

        let workflow_short_name = &expanded_task.workflow.short_name;
        let task_dir = task_root_dir
            .join(workflow_short_name)
            .join(task_id)
            .join(format!("call-{}", workflow_short_name));

        // TODO: In the current implementation, we don't support files on the cloud storage.
        let possible_file_paths = vec![
            task_dir.join(file_name),
            task_dir.join("execution").join(file_name),
            // TODO: add more possible file paths based on the cromwell doc
        ];

        for path in possible_file_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "File not found in the task directory: {}",
            task_id
        ))
    }
}

impl Task {
    pub fn update_owner(&mut self, owner: String) -> &Self {
        self.owner = owner;
        return self;
    }

    pub fn update_results(&mut self, results: Option<JsonValue>) -> &Self {
        self.results = results;
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Notification {
    #[serde(skip_deserializing)]
    #[oai(read_only)]
    id: i64,

    #[validate(length(
        max = "DEFAULT_LENGTH_255",
        min = "DEFAULT_LENGTH_1",
        message = "The length of title should be between 1 and 255."
    ))]
    title: String,

    description: Option<String>,

    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
    notification_type: String,

    #[serde(with = "ts_seconds")]
    created_time: DateTime<Utc>,

    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
    status: String, // "Unread" or "Read"

    #[validate(length(
        max = "DEFAULT_LENGTH_32",
        min = "DEFAULT_LENGTH_1",
        message = "The length of id should be between 1 and 32."
    ))]
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
