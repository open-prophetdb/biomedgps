use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{interval, sleep, timeout};

// Task configuration structure
struct TaskConfig {
    interval: Duration,
    retries: u32,
    timeout: Duration,
}

// Task information structure, including task name and execution function
struct Task {
    name: String,
    func: Arc<Box<dyn Fn() -> Result<(), String> + Send + Sync>>, // Task function, return Result
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
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
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
    pub async fn start_all(&self) {
        let mut handles = vec![];

        for task in self.tasks.values() {
            let task_clone = task.clone();
            handles.push(tokio::spawn(async move {
                run_task_loop(task_clone).await;
            }));
        }

        for handle in handles {
            let _ = handle.await;
        }
    }
}

// Task main loop, including congestion detection, retry, and timeout
pub async fn run_task_loop(task: Task) {
    let mut interval = interval(task.config.interval);

    loop {
        interval.tick().await; // Wait for the next time interval

        // Detect if the task is running to prevent congestion
        let mut running = task.is_running.lock().unwrap();
        if *running {
            println!("Task {} is running, skip this round", task.name);
            continue; // Skip task execution
        }
        *running = true;
        drop(running); // Release the lock

        // Start executing the task (retry mechanism)
        let task_func = Arc::clone(&task.func);
        let is_task_running = Arc::clone(&task.is_running);
        let task_name = task.name.clone();
        let max_retries = task.config.retries;
        let task_timeout = task.config.timeout;

        tokio::spawn(async move {
            let result =
                run_task_with_retry(&task_name, task_func, max_retries, task_timeout).await;

            if let Err(err) = result {
                println!("Task {} finally failed: {}", &task_name, err);
            }

            // Reset the status after the task is finished
            let mut running = is_task_running.lock().unwrap();
            *running = false;
        });
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
    F: Fn() -> Result<(), String> + Send + Sync + 'static,
{
    let mut attempts = 0;

    while attempts <= retries {
        println!("Task {} attempt {}", name, attempts + 1);

        // Use tokio::time::timeout to implement timeout control
        let func_clone = Arc::clone(&func);
        let task_future = tokio::spawn(async move { (func_clone)() });

        let result = timeout(timeout_duration, task_future).await;

        match result {
            Ok(Ok(Ok(()))) => {
                println!("Task {} success", name);
                return Ok(());
            }
            Ok(Ok(Err(e))) => {
                println!("Task {} failed: {}", name, e);
            }
            Ok(Err(join_err)) => {
                println!("Task {} execution error: {}", name, join_err);
            }
            Err(_) => {
                println!("Task {} timeout", name);
            }
        }

        attempts += 1;
        if attempts <= retries {
            println!("Waiting 5 seconds before retrying task {}", name);
            sleep(Duration::from_secs(5)).await;
        } else {
            println!("Task {} retry count exhausted", name);
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
