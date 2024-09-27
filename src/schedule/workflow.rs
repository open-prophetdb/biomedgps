use super::task_manager::TaskManager;
use crate::model::workspace::Task as WorkflowTask;
use cromwell_api_rs::CromwellClient;
use log::{error, info};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

pub async fn sync_task_status(
    pool: &Arc<PgPool>,
    cromwell_client_url: String,
) -> Result<(), anyhow::Error> {
    let sql_str = "SELECT * FROM biomedgps_tasks WHERE (status IS NULL OR (status != 'Failed' AND status != 'Succeeded')) AND finished_time IS NULL";

    let mut conn = pool.acquire().await?;
    let cromwell_client = CromwellClient::new(&cromwell_client_url);
    let tasks = sqlx::query_as::<_, WorkflowTask>(sql_str)
        .fetch_all(&mut conn)
        .await?;

    for task in tasks {
        info!("Syncing task status: {}", task.task_id);
        let task_id = task.task_id;

        let metadata = match cromwell_client.get_workflow_metadata(&task_id).await {
            Ok(metadata) => metadata,
            Err(e) => {
                error!("Failed to get workflow status: {}", e);
                return Err(anyhow::anyhow!(e));
            }
        };

        let status = metadata.status;
        let started_time = metadata.start;
        let finished_time = metadata.end;

        match sqlx::query("UPDATE biomedgps_tasks SET status = $1, started_time = $2, finished_time = $3 WHERE task_id = $4")
            .bind(status)
            .bind(started_time)
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
    let sql_str = "SELECT * FROM biomedgps_tasks WHERE (status IS NULL OR (status != 'Failed' AND status != 'Succeeded')) AND finished_time IS NULL";

    let mut conn = pool.acquire().await?;
    let cromwell_client = CromwellClient::new(&cromwell_client_url);
    let tasks = sqlx::query_as::<_, WorkflowTask>(sql_str)
        .fetch_all(&mut conn)
        .await?;

    for task in tasks {
        info!("Syncing log message: {}", task.task_id);
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
                        message.push_str("\nStdout:\n");
                        message.push_str(&std::fs::read_to_string(stdout).unwrap());
                    }

                    if std::path::Path::new(&stderr).exists() {
                        message.push_str("\nStderr:\n");
                        message.push_str(&std::fs::read_to_string(stderr).unwrap());
                    }
                }
                None => (),
            }
        }

        match sqlx::query("UPDATE biomedgps_tasks SET log_message = $1 WHERE task_id = $2")
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

pub async fn register_tasks(
    task_manager: &mut TaskManager,
    pool: &Arc<PgPool>,
    cromwell_client_url: &str,
) -> Result<(), anyhow::Error> {
    let pool1 = pool.clone();
    let cromwell_client_url1 = cromwell_client_url.to_string();
    // 注册任务 sync_task_status
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
        3,
        Duration::from_secs(10),
    );

    let pool2 = pool.clone();
    let cromwell_client_url2 = cromwell_client_url.to_string();
    // 注册任务 sync_log_message
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
        3,
        Duration::from_secs(15),
    );

    Ok(())
}
