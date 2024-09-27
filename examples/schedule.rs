use std::time::Duration;
use biomedgps::schedule::task_manager::TaskManager;

// 测试任务函数
async fn sample_task() -> Result<(), String> {
    println!("Executing sample task...");
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut manager = TaskManager::new();

    // Register task1: execute every 10 seconds, retry up to 3 times, timeout 5 seconds
    manager.register_task("task1", move || {
        let task = sample_task();
        tokio::spawn(async move {
            task.await
        })
    }, Duration::from_secs(10), 3, Duration::from_secs(5));

    // Register task2: execute every 15 seconds, retry up to 5 times, timeout 10 seconds
    manager.register_task("task2", move || {
        let task = sample_task();
        tokio::spawn(async move {
            task.await
        })
    }, Duration::from_secs(15), 5, Duration::from_secs(10));

    // Start all tasks
    manager.start_all().await;
}
