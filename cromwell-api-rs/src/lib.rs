use anyhow;
use log::{error, info};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

const VERSION: &str = "v1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StatusResponse {
    Empty(HashMap<String, String>),
    Detailed(HashMap<String, ServiceStatus>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub ok: bool,
    #[serde(default)]
    pub messages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    #[serde(rename = "workflowName")]
    pub workflow_name: Option<String>,
    #[serde(rename = "workflowProcessingEvents")]
    pub workflow_processing_events: Option<Vec<WorkflowProcessingEvent>>,
    #[serde(rename = "actualWorkflowLanguageVersion")]
    pub actual_workflow_language_version: Option<String>,
    #[serde(rename = "submittedFiles")]
    pub submitted_files: Option<SubmittedFiles>,
    pub calls: HashMap<String, Vec<CallMetadata>>,
    #[serde(rename = "outputs")]
    pub outputs: HashMap<String, serde_json::Value>,
    #[serde(rename = "workflowRoot")]
    pub workflow_root: Option<String>,
    #[serde(rename = "actualWorkflowLanguage")]
    pub actual_workflow_language: Option<String>,
    pub status: String,
    pub end: Option<String>,
    pub start: Option<String>,
    pub id: String,
    pub inputs: HashMap<String, serde_json::Value>,
    pub labels: HashMap<String, String>,
    pub submission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowProcessingEvent {
    #[serde(rename = "cromwellId")]
    pub cromwell_id: String,
    pub description: String,
    pub timestamp: String,
    #[serde(rename = "cromwellVersion")]
    pub cromwell_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmittedFiles {
    pub workflow: String,
    pub root: String,
    pub options: String,
    pub inputs: String,
    #[serde(rename = "workflowUrl")]
    pub workflow_url: String,
    pub labels: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallMetadata {
    #[serde(rename = "executionStatus")]
    pub execution_status: String,
    #[serde(rename = "stdout")]
    pub stdout: Option<String>,
    #[serde(rename = "backendStatus")]
    pub backend_status: Option<String>,
    #[serde(rename = "commandLine")]
    pub command_line: Option<String>,
    #[serde(rename = "shardIndex")]
    pub shard_index: Option<i32>,
    pub outputs: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "runtimeAttributes")]
    pub runtime_attributes: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "callCaching")]
    pub call_caching: Option<CallCaching>,
    pub inputs: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "returnCode")]
    pub return_code: Option<i32>,
    #[serde(rename = "jobId")]
    pub job_id: Option<String>,
    pub backend: Option<String>,
    pub end: Option<String>,
    pub start: Option<String>,
    pub stderr: Option<String>,
    #[serde(rename = "callRoot")]
    pub call_root: Option<String>,
    pub attempt: Option<i32>,
    #[serde(rename = "executionEvents")]
    pub execution_events: Option<Vec<ExecutionEvent>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallCaching {
    #[serde(rename = "allowResultReuse")]
    pub allow_result_reuse: bool,
    #[serde(rename = "effectiveCallCachingMode")]
    pub effective_call_caching_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionEvent {
    #[serde(rename = "startTime")]
    pub start_time: String,
    pub description: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFile {
    pub stderr: String,
    pub stdout: String,
    pub attempt: i32,
    #[serde(rename = "shardIndex")]
    pub shard_index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileNotReady {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileNotFound {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFileReady {
    pub calls: HashMap<String, Vec<LogFile>>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkflowLogFiles {
    // WorkflowLogFile must be the first variant for discriminating LogFileNotReady and WorkflowLogFile, because LogFileNotReady and LogFileNotFound have the same variant name "id".
    WorkflowLogFile(LogFileReady),
    LogFileNotReady(LogFileNotReady),
    LogFileNotFound(LogFileNotFound),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CromwellError {
    pub response_code: u16,    // 200, 404, 500
    pub response_type: String, // Success, NotFound, InternalServerError
    pub response_message: String,
    pub response_text: Option<String>,
}

impl std::fmt::Display for CromwellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.response_type, self.response_message)
    }
}

impl std::error::Error for CromwellError {}

pub struct CromwellClient {
    base_url: String,
    client: reqwest::Client,
}

impl CromwellClient {
    pub fn new(base_url: &str) -> Self {
        CromwellClient {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /* -------------------Engine API------------------- */
    /// Get the status of the Cromwell server
    pub async fn get_status(&self) -> Result<StatusResponse, CromwellError> {
        let url = format!("{}/engine/{}/status", self.base_url, VERSION);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        let status_response = match response.json::<StatusResponse>().await {
            Ok(status_response) => status_response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        Ok(status_response)
    }

    /// Get the version of the Cromwell server
    pub async fn get_version(&self) -> Result<String, CromwellError> {
        let url = format!("{}/engine/{}/version", self.base_url, VERSION);
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        let version_response: HashMap<String, String> = match response.json().await {
            Ok(version_response) => version_response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        Ok(version_response
            .get("cromwell")
            .cloned()
            .unwrap_or_default())
    }

    /* -------------------Workflow API------------------- */
    /// Submit a workflow to the Cromwell server
    pub async fn submit_workflow(
        &self,
        workflow_source: Option<&Path>,
        workflow_inputs: Option<&Path>,
        workflow_options: Option<&Path>,
        labels: Option<&Path>,
        workflow_dependencies: Option<&Path>,
        requested_workflow_id: Option<&str>,
    ) -> Result<WorkflowStatus, CromwellError> {
        let url = format!("{}/api/workflows/{}", self.base_url, VERSION);

        let mut form = Form::new();

        if let Some(source) = workflow_source {
            let file_content = match Self::read_file(source).await {
                Ok(file_content) => file_content,
                Err(e) => return Err(Self::handle_io_error(e)),
            };

            form = form.part(
                "workflowSource",
                Part::bytes(file_content)
                    .file_name(source.file_name().unwrap().to_str().unwrap().to_string()),
            );
        }

        if let Some(inputs) = workflow_inputs {
            let file_content = match Self::read_file(inputs).await {
                Ok(file_content) => file_content,
                Err(e) => return Err(Self::handle_io_error(e)),
            };

            form = form.part(
                "workflowInputs",
                Part::bytes(file_content)
                    .file_name(inputs.file_name().unwrap().to_str().unwrap().to_string()),
            );
        }

        if let Some(options) = workflow_options {
            let file_content = match Self::read_file(options).await {
                Ok(file_content) => file_content,
                Err(e) => return Err(Self::handle_io_error(e)),
            };

            form = form.part(
                "workflowOptions",
                Part::bytes(file_content)
                    .file_name(options.file_name().unwrap().to_str().unwrap().to_string()),
            );
        }

        if let Some(label_file) = labels {
            let file_content = match Self::read_file(label_file).await {
                Ok(file_content) => file_content,
                Err(e) => return Err(Self::handle_io_error(e)),
            };

            form = form.part(
                "labels",
                Part::bytes(file_content).file_name(
                    label_file
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
            );
        }

        if let Some(dependencies) = workflow_dependencies {
            let file_content = match Self::read_file(dependencies).await {
                Ok(file_content) => file_content,
                Err(e) => return Err(Self::handle_io_error(e)),
            };

            form = form.part(
                "workflowDependencies",
                Part::bytes(file_content).file_name(
                    dependencies
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ),
            );
        }

        if let Some(id) = requested_workflow_id {
            form = form.text("requestedWorkflowId", id.to_string());
        }

        let response = match self.client.post(&url).multipart(form).send().await {
            Ok(response) => response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        if response.status().is_success() {
            let submission_response: WorkflowStatus = match response.json().await {
                Ok(submission_response) => submission_response,
                Err(e) => return Err(Self::handle_error(e)),
            };
            Ok(submission_response)
        } else {
            Err(Self::handle_error(response.error_for_status().unwrap_err()))
        }
    }

    async fn read_file(path: &Path) -> Result<Vec<u8>, std::io::Error> {
        if let Ok(mut file) = File::open(path) {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            Ok(contents)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", path.display()),
            ))
        }
    }

    pub async fn get_workflow_status(
        &self,
        workflow_id: &str,
    ) -> Result<WorkflowStatus, CromwellError> {
        let url = format!(
            "{}/api/workflows/{}/{}/status",
            self.base_url, VERSION, workflow_id
        );
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        if response.status().is_success() {
            let response_text = match response.text().await {
                Ok(response_text) => response_text,
                Err(e) => return Err(Self::handle_error(e)),
            };

            let workflow_status: WorkflowStatus = match serde_json::from_str(&response_text) {
                Ok(workflow_status) => workflow_status,
                Err(e) => {
                    error!("response: {}", response_text);
                    return Err(Self::handle_parse_error(e, Some(response_text)));
                }
            };
            Ok(workflow_status)
        } else {
            Err(Self::handle_error(response.error_for_status().unwrap_err()))
        }
    }

    pub async fn get_workflow_metadata(
        &self,
        workflow_id: &str,
    ) -> Result<WorkflowMetadata, CromwellError> {
        let url = format!(
            "{}/api/workflows/{}/{}/metadata",
            self.base_url, VERSION, workflow_id
        );
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        if response.status().is_success() {
            let response_text = match response.text().await {
                Ok(response_text) => response_text,
                Err(e) => return Err(Self::handle_error(e)),
            };

            let workflow_metadata: WorkflowMetadata = match serde_json::from_str(&response_text) {
                Ok(workflow_metadata) => workflow_metadata,
                Err(e) => {
                    error!("response: {}", response_text);
                    return Err(Self::handle_parse_error(e, Some(response_text)));
                }
            };

            Ok(workflow_metadata)
        } else {
            Err(Self::handle_error(response.error_for_status().unwrap_err()))
        }
    }

    pub async fn get_workflow_logfiles(
        &self,
        workflow_id: &str,
    ) -> Result<LogFileReady, CromwellError> {
        let url = format!(
            "{}/api/workflows/{}/{}/logs",
            self.base_url, VERSION, workflow_id
        );
        let response = match self.client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => return Err(Self::handle_error(e)),
        };

        if response.status().is_success() {
            let response_text = match response.text().await {
                Ok(response_text) => response_text,
                Err(e) => return Err(Self::handle_error(e)),
            };

            let log_files: WorkflowLogFiles = match serde_json::from_str(&response_text) {
                Ok(log_files) => log_files,
                Err(e) => {
                    error!("response: {}", response_text);
                    return Err(Self::handle_parse_error(e, Some(response_text)));
                }
            };

            match log_files {
                WorkflowLogFiles::WorkflowLogFile(log_file) => Ok(log_file),
                WorkflowLogFiles::LogFileNotReady(log_file_not_ready) => {
                    return Err(CromwellError {
                        response_code: 202,
                        response_type: "NotReady".to_string(),
                        response_message: log_file_not_ready.id,
                        response_text: Some(response_text),
                    });
                }
                WorkflowLogFiles::LogFileNotFound(log_file_not_found) => {
                    return Err(CromwellError {
                        response_code: 404,
                        response_type: "NotFound".to_string(),
                        response_message: log_file_not_found.message,
                        response_text: Some(response_text),
                    });
                }
            }
        } else {
            Err(Self::handle_error(response.error_for_status().unwrap_err()))
        }
    }

    fn handle_error(e: reqwest::Error) -> CromwellError {
        let response_code = match e.status() {
            Some(status) => status.as_u16(),
            None => 500,
        };

        let response_type = match response_code {
            200 => "Success",
            404 => "NotFound",
            500 => "InternalServerError",
            _ => "Unknown",
        };

        return CromwellError {
            response_code: response_code,
            response_type: response_type.to_string(),
            response_message: e.to_string(),
            response_text: None,
        };
    }

    fn handle_parse_error(e: serde_json::Error, response_text: Option<String>) -> CromwellError {
        return CromwellError {
            response_code: 500,
            response_type: "InternalServerError".to_string(),
            response_message: e.to_string(),
            response_text: response_text,
        };
    }

    fn handle_io_error(e: std::io::Error) -> CromwellError {
        return CromwellError {
            response_code: 400,
            response_type: "BadRequest".to_string(),
            response_message: e.to_string(),
            response_text: None,
        };
    }
}

// Print the status of the Cromwell server
pub fn print_status(status: &StatusResponse) {
    match status {
        StatusResponse::Empty(_) => info!("Server Status: OK (Empty Response)"),
        StatusResponse::Detailed(services) => {
            info!("Server Status:");
            for (service, status) in services {
                info!("  {}: {}", service, if status.ok { "OK" } else { "ERROR" });
                if !status.messages.is_empty() {
                    info!("    Messages:");
                    for message in &status.messages {
                        info!("      - {}", message);
                    }
                }
            }
        }
    }
}

pub fn render_workflow_metadata(
    input_file: &Path,
    metadata: &serde_json::Value,
    dest_dir: &Path,
) -> Result<PathBuf, anyhow::Error> {
    if !dest_dir.exists() {
        return Err(anyhow::anyhow!(
            "Destination directory does not exist: {}",
            dest_dir.display()
        ));
    }

    let dir = match input_file.parent() {
        Some(dir) => dir,
        None => {
            return Err(anyhow::anyhow!(
                "No parent directory for input file: {}",
                input_file.display()
            ))
        }
    };

    let tera = match Tera::new(dir.join("*.json").to_str().unwrap()) {
        Ok(tera) => tera,
        Err(e) => return Err(anyhow::anyhow!("Failed to create Tera instance: {}", e)),
    };

    let mut context = Context::new();
    context.insert("metadata", metadata);

    let filename = input_file.file_name().unwrap();
    let dest_file = dest_dir.join(filename);
    match tera.render(filename.to_str().unwrap(), &context) {
        Ok(rendered) => {
            std::fs::write(&dest_file, rendered)?;
            Ok(dest_file)
        }
        Err(e) => return Err(anyhow::anyhow!("Failed to render template: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_status() {
        let client = CromwellClient::new("http://localhost:8001");
        let status = client.get_status().await;
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_get_version() {
        let client = CromwellClient::new("http://localhost:8001");
        let version = client.get_version().await;
        assert!(version.is_ok());
    }
}
