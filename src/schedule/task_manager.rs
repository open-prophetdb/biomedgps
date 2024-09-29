use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use tokio::{
    sync::Notify,
    time::{interval, sleep, timeout, Duration},
};
use log::{error, info, warn};

// Task configuration structure
struct TaskConfig {
    interval: Duration,
    retries: u32,
    timeout: Duration,
}

// Task information structure, including task name and execution function
pub struct Task {
    name: String,
    func: Arc<Box<dyn Fn() -> JoinHandle<Result<(), String>> + Send + Sync>>, // Task function, return Result
    config: TaskConfig,
    is_running: Arc<Mutex<bool>>, // Record if the task is running
}

// Task manager, for registering and managing tasks
pub struct TaskManager {
    tasks: HashMap<String, Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager {
            tasks: HashMap::new(),
        }
    }

    // Register task
    pub fn register_task<F>(
        &mut self,
        name: &str,
        func: F,
        interval: Duration,
        retries: u32,
        timeout: Duration,
    ) where
        F: Fn() -> JoinHandle<Result<(), String>> + Send + Sync + 'static,
    {
        let task = Task {
            name: name.to_string(),
            func: Arc::new(Box::new(func)),
            config: TaskConfig {
                interval,
                retries,
                timeout,
            },
            is_running: Arc::new(Mutex::new(false)),
        };
        self.tasks.insert(name.to_string(), task);
    }

    // Start all tasks
    pub async fn start_all(&self, shutdown_notify: Arc<Notify>) {
        let mut handles = vec![];

        for task in self.tasks.values() {
            let task_clone = task.clone();
            let shutdown_notify_clone = Arc::clone(&shutdown_notify);

            handles.push(tokio::spawn(async move {
                run_task_loop(task_clone, shutdown_notify_clone).await;
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }
}

// Task main loop, including congestion detection, retry, and timeout
pub async fn run_task_loop(task: Task, shutdown_notify: Arc<Notify>) {
    let mut interval = interval(task.config.interval);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // 检测任务是否正在运行，防止拥塞
                let mut running = task.is_running.lock().unwrap();
                if *running {
                    info!("Task {} is running, skip this round", task.name);
                    continue; // 跳过任务执行
                }
                *running = true;
                drop(running); // 释放锁

                // 开始执行任务（包含重试机制）
                let task_func = Arc::clone(&task.func);
                let is_task_running = Arc::clone(&task.is_running);
                let task_name = task.name.clone();
                let max_retries = task.config.retries;
                let task_timeout = task.config.timeout;

                tokio::spawn(async move {
                    let result =
                        run_task_with_retry(&task_name, task_func, max_retries, task_timeout).await;

                    if let Err(err) = result {
                        info!("Task {} finally failed: {}", &task_name, err);
                    }

                    // 任务结束后重置状态
                    let mut running = is_task_running.lock().unwrap();
                    *running = false;
                });
            },
            _ = shutdown_notify.notified() => {
                info!("Received shutdown signal, stopping task {}", task.name);
                break; // 跳出循环，停止任务
            }
        }
    }
}

// Execute a single task and retry according to the configuration
pub async fn run_task_with_retry<F>(
    name: &str,
    func: Arc<F>,
    retries: u32,
    timeout_duration: Duration,
) -> Result<(), String>
where
    F: Fn() -> JoinHandle<Result<(), String>> + Send + Sync + 'static,
{
    let mut attempts = 0;

    while attempts <= retries {
        info!("Task {} attempt {}", name, attempts + 1);

        // Use tokio::time::timeout to implement timeout control
        let func_clone = Arc::clone(&func);
        let task_future = tokio::spawn(async move { (func_clone)().await });

        let result = timeout(timeout_duration, task_future).await;

        match result {
            Ok(Ok(Ok(_))) => {
                info!("Task {} success", name);
                return Ok(());
            }
            Ok(Ok(Err(e))) => {
                error!("Task {} failed: {}", name, e);
            }
            Ok(Err(join_err)) => {
                error!("Task {} execution error: {}", name, join_err);
            }
            Err(_) => {
                error!("Task {} timeout", name);
            }
        }

        attempts += 1;
        if attempts <= retries {
            warn!("Waiting 5 seconds before retrying task {}", name);
            sleep(Duration::from_secs(5)).await;
        } else {
            warn!("Task {} retry count exhausted", name);
            return Err("Task retry failed".to_string());
        }
    }

    Err("Task retry failed".to_string())
}

// 任务结构体的 Clone 实现（用于多线程环境）
impl Clone for Task {
    fn clone(&self) -> Self {
        Task {
            name: self.name.clone(),
            func: Arc::clone(&self.func),
            config: TaskConfig {
                interval: self.config.interval,
                retries: self.config.retries,
                timeout: self.config.timeout,
            },
            is_running: Arc::clone(&self.is_running),
        }
    }
}
