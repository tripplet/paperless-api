#[cfg(test)]
mod tests {
    use std::{
        env,
        error::Error,
        io,
        path::PathBuf,
        process,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };

    use paperless_api::{
        PaperlessClient,
        attributes::{Tag, tag::CreateTag},
        id::DocumentId,
        task::TaskStatus,
    };
    use serde::Deserialize;
    use tokio::sync::OnceCell;

    type TestError = Box<dyn Error + Send + Sync>;
    type TestResult<T = ()> = Result<T, TestError>;

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

    struct TestContext {
        client: PaperlessClient,
        document_path: PathBuf,
        task_timeout: Duration,
        poll_interval: Duration,
    }

    #[tokio::test]
    async fn ordered_integration_suite() -> TestResult {
        let context = context().await?;

        // Keep stateful steps ordered here; the Rust test harness does not order test cases.
        upload_document(context).await?;
        tag_lifecycle(context).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore = "debug helper; run explicitly with --ignored"]
    async fn debug_upload_document() -> TestResult {
        upload_document(context().await?).await
    }

    #[tokio::test]
    #[ignore = "debug helper; run explicitly with --ignored"]
    async fn debug_tag_lifecycle() -> TestResult {
        tag_lifecycle(context().await?).await
    }

    async fn context() -> TestResult<&'static TestContext> {
        CONTEXT
            .get_or_try_init(|| async {
                let config = load_config()?;
                let token = match config.token.as_deref() {
                    Some(token) => token.to_owned(),
                    None => {
                        let username = required(&config.username, "username")?;
                        let password = required(&config.password, "password")?;
                        PaperlessClient::request_token(
                            &config.base_url,
                            None,
                            None,
                            username,
                            password,
                        )
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
                .unwrap_or_else(|| std::path::Path::new("."))
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

    async fn upload_document(context: &TestContext) -> TestResult {
        let filename = format!("paperless-api-integration-{}.pdf", unique_suffix());
        let task_id = context
            .client
            .upload_document(&context.document_path, &filename)
            .await?;
        let document_id = wait_for_document(context, &task_id).await?;

        let verification = async {
            let document = context
                .client
                .get_document_by_id(document_id, None, None)
                .await?;
            assert_eq!(document.id(), document_id);
            assert_eq!(document.original_file_name(), filename);
            assert_eq!(document.mime_type(), Some("application/pdf"));
            Ok::<(), TestError>(())
        }
        .await;

        let cleanup = delete_document(&context.client, document_id).await;
        verification.and(cleanup)
    }

    async fn wait_for_document(
        context: &TestContext,
        task_id: &paperless_api::id::UniqueTaskId,
    ) -> TestResult<DocumentId> {
        let deadline = Instant::now() + context.task_timeout;

        loop {
            let tasks = context
                .client
                .get_task_status(Some(task_id), None, None)
                .await?;

            if let Some(task) = tasks.into_iter().find(|task| &task.task_id == task_id) {
                match task.status {
                    TaskStatus::Success => {
                        return task
                            .related_documents
                            .and_then(|documents| documents.into_iter().next())
                            .ok_or_else(|| {
                                io::Error::other(format!(
                                    "upload task {task_id} succeeded without a document ID"
                                ))
                                .into()
                            });
                    }
                    TaskStatus::Failure | TaskStatus::Revoked => {
                        return Err(io::Error::other(format!(
                            "upload task {task_id} ended with {}: {}",
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
                    format!("upload task {task_id} did not finish in time"),
                )
                .into());
            }

            tokio::time::sleep(context.poll_interval).await;
        }
    }

    async fn delete_document(client: &PaperlessClient, document_id: DocumentId) -> TestResult {
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

    async fn tag_lifecycle(context: &TestContext) -> TestResult {
        let existing_tags = context.client.load_items::<Tag>().await?;
        let name = format!("paperless-api-integration-{}", unique_suffix());
        assert!(existing_tags.values().all(|tag| tag.name != name));

        let created = context
            .client
            .create(&CreateTag {
                name: name.clone(),
                color: "#3366cc".to_owned(),
                ..Default::default()
            })
            .await?;

        let verification = async {
            let tags = context.client.load_items::<Tag>().await?;
            let loaded = tags
                .get(&created.id)
                .ok_or_else(|| io::Error::other("created tag was not returned by the tags API"))?;
            assert_eq!(loaded.name, name);
            Ok::<(), TestError>(())
        }
        .await;

        let cleanup = context.client.delete(created.id).await.map_err(Into::into);
        verification.and(cleanup)?;
        assert!(
            context
                .client
                .load_by_id::<Tag>(created.id)
                .await?
                .is_none(),
            "deleted tag is still available"
        );

        Ok(())
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
}
