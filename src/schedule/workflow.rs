use super::task_manager::TaskManager;
use crate::model::workspace::Task as WorkflowTask;
use crate::model::workspace::{Task, Workflow};
use cromwell_api_rs::{render_workflow_metadata, CromwellClient};
use log::{debug, error, info};
use sqlx::PgPool;
use sqlx::Row;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub async fn sync_task_status(
    pool: &Arc<PgPool>,
    cromwell_client_url: String,
) -> Result<(), anyhow::Error> {
    // started_time and status are null, means the workflow is not submitted to Cromwell
    // finished_time is null and started_time is not null, means the workflow is submitted to Cromwell but not finished
    let sql_str =
        "SELECT * FROM biomedgps_task WHERE finished_time IS NULL AND started_time IS NOT NULL";

    let mut conn = pool.acquire().await?;
    let cromwell_client = CromwellClient::new(&cromwell_client_url);
    let tasks = sqlx::query_as::<_, WorkflowTask>(sql_str)
        .fetch_all(&mut conn)
        .await?;

    for task in tasks {
        debug!("Syncing task status: {}", task.task_id);
        let task_id = task.task_id;

        let metadata = match cromwell_client.get_workflow_metadata(&task_id).await {
            Ok(metadata) => metadata,
            Err(e) => {
                error!("Failed to get workflow status: {}", e);
                return Err(anyhow::anyhow!(e));
            }
        };

        let status = metadata.status;
        let finished_time = metadata.end;

        match sqlx::query("UPDATE biomedgps_task SET status = $1, finished_time = $2::timestamptz WHERE task_id = $3")
            .bind(status)
            .bind(finished_time)
            .bind(task_id)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update task status: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn sync_log_message(
    pool: &Arc<PgPool>,
    cromwell_client_url: String,
) -> Result<(), anyhow::Error> {
    let sql_str =
        "SELECT * FROM biomedgps_task WHERE (finished_time IS NULL AND started_time IS NOT NULL) OR log_message IS NULL";

    let mut conn = pool.acquire().await?;
    let cromwell_client = CromwellClient::new(&cromwell_client_url);
    let tasks = sqlx::query_as::<_, WorkflowTask>(sql_str)
        .fetch_all(&mut conn)
        .await?;

    for task in tasks {
        debug!("Syncing log message: {}", task.task_id);
        let task_id = task.task_id;

        let log = match cromwell_client.get_workflow_logfiles(&task_id).await {
            Ok(log_files) => log_files,
            Err(e) => {
                error!("Failed to get log message: {}", e);
                return Err(anyhow::anyhow!(e));
            }
        };

        let mut message = String::new();
        for key in log.calls.keys() {
            let logs = log.calls.get(key).unwrap();
            let first_log = logs.first();

            match first_log {
                Some(logmap) => {
                    message.push_str(&format!("Task: {}\n", key));

                    let stderr = logmap.stdout.clone();
                    let stdout = logmap.stderr.clone();

                    if std::path::Path::new(&stdout).exists() {
                        message.push_str("\n\nStdout:\n");
                        message.push_str(&std::fs::read_to_string(stdout).unwrap());
                    }

                    if std::path::Path::new(&stderr).exists() {
                        message.push_str("\n\nStderr:\n");
                        message.push_str(&std::fs::read_to_string(stderr).unwrap());
                    }
                }
                None => (),
            }
        }

        match sqlx::query("UPDATE biomedgps_task SET log_message = $1 WHERE task_id = $2")
            .bind(message)
            .bind(task_id)
            .execute(&mut conn)
            .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update log message: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn submit_workflow(
    pool: &Arc<PgPool>,
    cromwell_client_url: &str,
) -> Result<(), anyhow::Error> {
    let cromwell_client = CromwellClient::new(cromwell_client_url);

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
        JOIN biomedgps_workflow ON biomedgps_task.workflow_id = biomedgps_workflow.id 
        WHERE biomedgps_task.status IS NULL AND biomedgps_task.started_time IS NULL
    ";
    let mut conn = pool.acquire().await?;
    let rows = match sqlx::query(sql_str).fetch_all(&mut conn).await {
        Ok(rows) => rows,
        Err(sqlx::Error::RowNotFound) => return Err(anyhow::anyhow!("No task or workflow found")),
        Err(e) => return Err(anyhow::anyhow!("Failed to get expanded task: {}", e)),
    };

    for row in rows {
        let task = Task {
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

        let workflow_root_dir = match std::env::var("WORKFLOW_ROOT_DIR") {
            Ok(workflow_root_dir) => PathBuf::from(workflow_root_dir),
            Err(e) => {
                error!(
                    "The WORKFLOW_ROOT_DIR environment variable is not set: {}",
                    e
                );
                return Err(anyhow::anyhow!(e));
            }
        };

        let workflow_dir = Workflow::get_workflow_installation_path(
            &workflow_root_dir,
            &Workflow::get_workflow_dirname(&workflow.short_name, &workflow.version),
        );
        let task_template_dir = Workflow::get_task_template_dir(&workflow_root_dir, &task.task_id);
        let input_json_file = workflow_dir.join("inputs.json");

        if std::path::Path::new(&input_json_file).exists() {
            let task_params = task.task_params;

            if !task_template_dir.exists() {
                // Create task template directory
                std::fs::create_dir_all(&task_template_dir)?;
            }

            match render_workflow_metadata(&input_json_file, &task_params, &workflow_dir.to_str().unwrap(), &task_template_dir, false) {
                Ok(rendered_input_json_file) => {
                    let workflow_file = workflow_dir.join("workflow.wdl");

                    match cromwell_client
                        .submit_workflow(
                            Some(workflow_file.as_path()),
                            Some(rendered_input_json_file.as_path()),
                            None,
                            None,
                            None,
                            Some(&task.task_id),
                        )
                        .await
                    {
                        Ok(workflow_status) => {
                            debug!("Workflow submitted: {:?}", workflow_status);
                        }
                        Err(e) => {
                            if e.response_type == "DuplicateWorkflowId" {
                                // We assume the workflow id is globally unique, so if the workflow id is duplicated, the submission is successful
                                debug!("Workflow submitted: {:?}", e);
                            } else {
                                error!("Failed to submit workflow: {:?}", e);
                                return Err(anyhow::anyhow!(e));
                            };
                        }
                    }

                    match sqlx::query("UPDATE biomedgps_task SET status = $1, started_time = $2::timestamptz WHERE task_id = $3")
                        .bind("Submitted")
                        .bind(chrono::Utc::now())
                        .bind(task.task_id)
                        .execute(&mut conn)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Failed to update task status: {:?}", e);
                            // TODO: Rollback the workflow submission
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to render workflow metadata: {}", e);
                    return Err(anyhow::anyhow!(e));
                }
            }
        } else {
            error!(
                "Input json file not found at {}, it might be a invalid workflow.",
                input_json_file.display()
            );
        }
    }

    Ok(())
}

pub async fn register_tasks(
    task_manager: &mut TaskManager,
    pool: &Arc<PgPool>,
    cromwell_client_url: &str,
) -> Result<(), anyhow::Error> {
    let pool1 = pool.clone();
    let cromwell_client_url1 = cromwell_client_url.to_string();
    // Register task sync_task_status
    task_manager.register_task(
        "sync_task_status",
        move || {
            let pool_clone = pool1.clone();
            let cromwell_client_url_clone = cromwell_client_url1.clone();

            tokio::spawn(async move {
                match sync_task_status(&pool_clone, cromwell_client_url_clone).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Failed to sync task status: {}", e);
                        Err(e.to_string())
                    }
                }
            })
        },
        Duration::from_secs(5),
        0,
        Duration::from_secs(10),
    );

    let pool2 = pool.clone();
    let cromwell_client_url2 = cromwell_client_url.to_string();
    // Register task sync_log_message
    task_manager.register_task(
        "sync_log_message",
        move || {
            let pool_clone = pool2.clone();
            let cromwell_client_url_clone = cromwell_client_url2.clone();

            tokio::spawn(async move {
                match sync_log_message(&pool_clone, cromwell_client_url_clone).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Failed to sync log message: {}", e);
                        Err(e.to_string())
                    }
                }
            })
        },
        Duration::from_secs(10),
        0,
        Duration::from_secs(15),
    );

    let pool3 = pool.clone();
    let cromwell_client_url3 = cromwell_client_url.to_string();
    // Register task submit_workflow
    task_manager.register_task(
        "submit_workflow",
        move || {
            let pool_clone = pool3.clone();
            let cromwell_client_url_clone = cromwell_client_url3.clone();

            tokio::spawn(async move {
                match submit_workflow(&pool_clone, &cromwell_client_url_clone).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Failed to submit workflow: {}", e);
                        Err(e.to_string())
                    }
                }
            })
        },
        Duration::from_secs(3),
        0,
        Duration::from_secs(8),
    );

    Ok(())
}
