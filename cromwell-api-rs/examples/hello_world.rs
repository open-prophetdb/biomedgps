use cromwell_api_rs::{print_status, CromwellClient, CromwellError};
use env_logger;
use log::{error, info};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), CromwellError> {
    let _ = env_logger::builder().is_test(true).try_init();

    let client = CromwellClient::new("http://localhost:8000");
    let status = match client.get_status().await {
        Ok(status) => status,
        Err(e) => {
            eprintln!("获取服务器状态失败: {}", e);

            return Err(e);
        }
    };

    print_status(&status);

    let version = match client.get_version().await {
        Ok(version) => version,
        Err(e) => {
            eprintln!("获取服务器版本失败: {}", e);
            return Err(e);
        }
    };

    println!("服务器版本: {}", version);

    let workflow_uuid = uuid::Uuid::new_v4().to_string();
    let workflow_source = Some(Path::new("examples/hello_world/workflow.wdl"));
    let workflow_inputs = Some(Path::new("examples/hello_world/intputs.json"));

    let status = match client
        .submit_workflow(
            workflow_source,
            workflow_inputs,
            None,
            None,
            None,
            Some(&workflow_uuid),
        )
        .await
    {
        Ok(status) => status,
        Err(e) => {
            eprintln!("提交工作流失败: {}", e);
            return Err(e);
        }
    };

    println!("提交工作流{}成功: {:?}", workflow_uuid, status);

    // Sleep for 20 seconds to allow the workflow to complete
    tokio::time::sleep(std::time::Duration::from_secs(20)).await;

    let status = match client.get_workflow_status(&workflow_uuid).await {
        Ok(status) => status,
        Err(e) => {
            eprintln!("获取工作流状态失败: {}", e);
            return Err(e);
        }
    };

    println!("工作流{}的状态: {:?}", workflow_uuid, status);

    let metadata = match client.get_workflow_metadata(&workflow_uuid).await {
        Ok(metadata) => metadata,
        Err(e) => {
            eprintln!("获取工作流元数据失败: {}", e);
            return Err(e);
        }
    };

    println!("工作流{}的元数据: {:?}", workflow_uuid, metadata);

    // let workflow_uuid = "97d38066-a9db-49fb-a100-ac90255f7247";
    let logfiles = match client.get_workflow_logfiles(&workflow_uuid).await {
        Ok(logfiles) => logfiles,
        Err(e) => {
            eprintln!("获取工作流日志文件失败: {}", e);
            return Err(e);
        }
    };

    println!("工作流{}的日志文件: {:?}", workflow_uuid, logfiles);

    Ok(())
}
