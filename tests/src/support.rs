use std::{
    env,
    error::Error,
    io,
    path::{Path, PathBuf},
    process,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use paperless_api::{
    PaperlessClient,
    id::{DocumentId, UniqueTaskId},
    task::{Task, TaskStatus},
};
use serde::Deserialize;
use tokio::sync::OnceCell;

pub(crate) type TestError = Box<dyn Error + Send + Sync>;
pub(crate) type TestResult<T = ()> = Result<T, TestError>;

static CONTEXT: OnceCell<TestContext> = OnceCell::const_new();

#[derive(Deserialize)]
struct TestConfig {
    base_url: String,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
    document_path: PathBuf,
    #[serde(default = "default_task_timeout_seconds")]
    task_timeout_seconds: u64,
    #[serde(default = "default_poll_interval_milliseconds")]
    poll_interval_milliseconds: u64,
}

pub(crate) struct TestContext {
    pub(crate) client: PaperlessClient,
    pub(crate) document_path: PathBuf,
    task_timeout: Duration,
    poll_interval: Duration,
}

pub(crate) async fn context() -> TestResult<&'static TestContext> {
    CONTEXT
        .get_or_try_init(|| async {
            let config = load_config()?;
            let token = match config.token.as_deref() {
                Some(token) => token.to_owned(),
                None => {
                    let username = required(&config.username, "username")?;
                    let password = required(&config.password, "password")?;
                    PaperlessClient::request_token(&config.base_url, None, None, username, password)
                        .await?
                }
            };

            Ok::<TestContext, TestError>(TestContext {
                client: PaperlessClient::new(&config.base_url, &token, None)?,
                document_path: config.document_path,
                task_timeout: Duration::from_secs(config.task_timeout_seconds),
                poll_interval: Duration::from_millis(config.poll_interval_milliseconds),
            })
        })
        .await
}

pub(crate) async fn wait_for_task(
    context: &TestContext,
    task_id: &UniqueTaskId,
) -> TestResult<Task> {
    let deadline = Instant::now() + context.task_timeout;

    loop {
        let tasks = context
            .client
            .get_task_status(Some(task_id), None, None)
            .await?;

        if let Some(task) = tasks.into_iter().find(|task| &task.task_id == task_id) {
            match task.status {
                TaskStatus::Success => return Ok(task),
                TaskStatus::Failure | TaskStatus::Revoked => {
                    return Err(io::Error::other(format!(
                        "task {task_id} ended with {}: {}",
                        task.status,
                        task.result.as_deref().unwrap_or("no result")
                    ))
                    .into());
                }
                TaskStatus::Pending | TaskStatus::Started => {}
            }
        }

        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("task {task_id} did not finish in time"),
            )
            .into());
        }

        tokio::time::sleep(context.poll_interval).await;
    }
}

pub(crate) fn document_id(task: &Task) -> TestResult<DocumentId> {
    task.related_documents
        .as_ref()
        .and_then(|documents| documents.first().copied())
        .ok_or_else(|| {
            io::Error::other(format!(
                "task {} succeeded without a document ID",
                task.task_id
            ))
            .into()
        })
}

pub(crate) async fn delete_document(
    client: &PaperlessClient,
    document_id: DocumentId,
) -> TestResult {
    if let Some(mut document) = client
        .get_document_by_id(document_id, None, None)
        .await
        .map(Some)
        .or_else(|error| match error {
            paperless_api::Error::NotFound => Ok(None),
            error => Err(error),
        })?
    {
        document.delete().await?;
    }
    Ok(())
}

pub(crate) fn temporary_path(extension: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "paperless-api-integration-{}.{}",
        unique_suffix(),
        extension
    ))
}

pub(crate) fn unique_name(kind: &str) -> String {
    format!("paperless-api-{kind}-{}", unique_suffix())
}

fn load_config() -> TestResult<TestConfig> {
    let path = env::var_os("PAPERLESS_TEST_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.json"));
    let contents = std::fs::read_to_string(&path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("failed to read test config {}: {error}", path.display()),
        )
    })?;
    let mut config: TestConfig = serde_json::from_str(&contents)?;
    if config.document_path.is_relative() {
        config.document_path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&config.document_path);
    }
    if !config.document_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "configured test document does not exist: {}",
                config.document_path.display()
            ),
        )
        .into());
    }

    Ok(config)
}

fn required<'a>(value: &'a Option<String>, field: &str) -> TestResult<&'a str> {
    value.as_deref().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("test config requires '{field}' when no token is provided"),
        )
        .into()
    })
}

fn unique_suffix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{timestamp}", process::id())
}

const fn default_task_timeout_seconds() -> u64 {
    90
}

const fn default_poll_interval_milliseconds() -> u64 {
    500
}
