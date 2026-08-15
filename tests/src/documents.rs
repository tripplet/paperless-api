use std::{fs, io};

use paperless_api::{document_query::DocumentQueryBuilder, task::TaskStatus};

use crate::support::{self, TestContext, TestError, TestResult};

pub(crate) async fn document_lifecycle(context: &TestContext) -> TestResult {
    let filename = format!("{}.pdf", support::unique_name("document"));
    let task_id = context
        .client
        .upload_document(&context.document_path, &filename)
        .await?;
    let task = support::wait_for_task(context, &task_id).await?;
    let document_id = support::document_id(&task)?;

    let second_filename = format!("{}.pdf", support::unique_name("second-document"));
    let second_task_id = context
        .client
        .upload_document(&context.second_document_path, &second_filename)
        .await?;
    let second_task = support::wait_for_task(context, &second_task_id).await?;
    let second_document_id = support::document_id(&second_task)?;

    let verification = async {
        assert_eq!(task.status, TaskStatus::Success);
        assert_eq!(task.task_id, task_id);
        assert_eq!(second_task.status, TaskStatus::Success);
        assert_eq!(second_task.task_id, second_task_id);

        let loaded_task = context
            .client
            .get_task_by_id(&task.id)
            .await?
            .ok_or_else(|| io::Error::other("upload task was not returned by ID"))?;
        assert_eq!(loaded_task.task_id, task_id);
        let loaded_second_task = context
            .client
            .get_task_by_id(&second_task.id)
            .await?
            .ok_or_else(|| io::Error::other("second upload task was not returned by ID"))?;
        assert_eq!(loaded_second_task.task_id, second_task_id);

        let mut document = context
            .client
            .get_document_by_id(document_id, Some(true), None)
            .await?;
        assert_eq!(document.id(), document_id);
        assert_eq!(document.original_file_name(), filename);
        assert_eq!(document.mime_type(), Some("application/pdf"));
        assert!(
            document
                .content()
                .as_ref()
                .contains("paperless-api-demo-1-document")
        );
        assert!(
            document
                .page_link()
                .ends_with(&format!("/documents/{document_id}/"))
        );

        let source = fs::read(&context.document_path)?;
        assert_eq!(document.download_to_buffer(true).await?, source);
        let processed = document.download_to_buffer(false).await?;
        assert!(processed.starts_with(b"%PDF-"));

        let second_document = context
            .client
            .get_document_by_id(second_document_id, Some(true), None)
            .await?;
        assert_eq!(second_document.id(), second_document_id);
        assert_eq!(second_document.original_file_name(), second_filename);
        assert_eq!(second_document.mime_type(), Some("application/pdf"));
        assert!(
            second_document
                .content()
                .as_ref()
                .contains("paperless-api-demo-2-document")
        );
        assert!(
            second_document
                .page_link()
                .ends_with(&format!("/documents/{second_document_id}/"))
        );

        let second_source = fs::read(&context.second_document_path)?;
        assert_eq!(
            second_document.download_to_buffer(true).await?,
            second_source
        );

        let original_download_path = support::temporary_path("pdf");
        document
            .download_to_file(&original_download_path, true)
            .await?;
        let original_downloaded = fs::read(&original_download_path);
        let _ = fs::remove_file(&original_download_path);
        assert_eq!(original_downloaded?, source);

        let processed_download_path = support::temporary_path("pdf");
        document
            .download_to_file(&processed_download_path, false)
            .await?;
        let processed_downloaded = fs::read(&processed_download_path);
        let _ = fs::remove_file(&processed_download_path);
        assert_eq!(processed_downloaded?, processed);

        let thumbnail = document.thumbnail().await?;
        assert!(!thumbnail.is_empty());

        let updated_title = support::unique_name("updated-title");
        document.set_title(updated_title.clone());
        assert!(document.is_dirty());
        document.patch().await?;
        assert!(!document.is_dirty());
        document.refresh().await?;
        assert_eq!(document.title(), updated_title);

        let documents = context
            .client
            .query_documents(DocumentQueryBuilder::default())
            .await?;
        assert!(documents.iter().any(|item| item.id() == document_id));
        assert!(documents.iter().any(|item| item.id() == second_document_id));

        context
            .client
            .acknowledge_tasks(&[task.id, second_task.id], false)
            .await?;
        let acknowledged = context
            .client
            .get_task_by_id(&task.id)
            .await?
            .ok_or_else(|| io::Error::other("acknowledged task was not returned by ID"))?;
        assert!(acknowledged.acknowledged);
        let second_acknowledged = context
            .client
            .get_task_by_id(&second_task.id)
            .await?
            .ok_or_else(|| io::Error::other("second acknowledged task was not returned by ID"))?;
        assert!(second_acknowledged.acknowledged);

        Ok::<(), TestError>(())
    }
    .await;

    let first_cleanup = support::delete_document(&context.client, document_id).await;
    let second_cleanup = support::delete_document(&context.client, second_document_id).await;
    verification.and(first_cleanup).and(second_cleanup)?;
    assert!(
        context
            .client
            .get_document_by_id(document_id, None, None)
            .await
            .is_err()
    );
    assert!(
        context
            .client
            .get_document_by_id(second_document_id, None, None)
            .await
            .is_err()
    );

    Ok(())
}

#[tokio::test]
#[ignore = "debug helper; run explicitly with --ignored"]
async fn debug_document_lifecycle() -> TestResult {
    document_lifecycle(support::context().await?).await
}
