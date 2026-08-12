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
            .get_document_by_id(document_id, None, Some(true), None)
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
        assert!(document.preview().await?.starts_with(b"%PDF-"));

        let metadata = document.get_metadata().await?;
        assert!(!metadata.original_checksum.is_empty());
        assert_eq!(
            metadata.original_filename.as_deref(),
            Some(filename.as_str())
        );
        assert_eq!(metadata.original_size, source.len() as u64);
        assert_eq!(
            metadata.original_mime_type.as_deref(),
            Some("application/pdf")
        );

        let second_document = context
            .client
            .get_document_by_id(second_document_id, None, Some(true), None)
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

        // Appending a PDF comment keeps the fixture valid while changing its checksum.
        let version_path = support::temporary_path("pdf");
        let version_filename = format!("{}.pdf", support::unique_name("document-version"));
        let initial_version_label = "initial integration version";
        let mut version_source = source.clone();
        version_source.extend_from_slice(
            format!("\n% {}\n", support::unique_name("version-marker")).as_bytes(),
        );
        fs::write(&version_path, &version_source)?;
        let version_task_id_result = document
            .upload_new_version(
                &version_path,
                &version_filename,
                Some(initial_version_label),
            )
            .await;
        fs::remove_file(&version_path)?;
        let version_task_id = version_task_id_result?;
        let version_task = support::wait_for_task(context, &version_task_id).await?;
        assert_eq!(version_task.status, TaskStatus::Success);
        assert_eq!(version_task.task_id, version_task_id);

        let version_id = support::document_version_id(&version_task)?;
        assert_ne!(version_id.as_document_id(), document_id);
        assert_eq!(
            context.client.find_root_document(version_id).await?,
            document_id
        );

        document.refresh().await?;
        let uploaded_version = document
            .versions()
            .iter()
            .find(|version| version.id == version_id)
            .ok_or_else(|| io::Error::other("uploaded version was not returned"))?;
        assert_eq!(
            uploaded_version.label.as_deref(),
            Some(initial_version_label)
        );
        let original_version_id = document
            .versions()
            .iter()
            .find(|version| version.is_root)
            .map(|version| version.id)
            .ok_or_else(|| io::Error::other("root version was not returned"))?;
        assert_eq!(original_version_id.as_document_id(), document_id);
        assert_eq!(
            context
                .client
                .find_root_document(original_version_id)
                .await?,
            document_id
        );

        // With no explicit selector, version-aware operations resolve to the latest version.
        assert_eq!(document.download_to_buffer(true).await?, version_source);

        let mut version_document = context
            .client
            .get_document_by_id(document_id, Some(version_id), Some(true), None)
            .await?;
        assert_eq!(version_document.id(), document_id);
        assert_eq!(version_document.selected_version(), Some(version_id));
        assert_eq!(
            version_document.download_to_buffer(true).await?,
            version_source
        );
        assert!(!version_document.thumbnail().await?.is_empty());
        assert!(version_document.preview().await?.starts_with(b"%PDF-"));

        let version_download_path = support::temporary_path("pdf");
        version_document
            .download_to_file(&version_download_path, true)
            .await?;
        let version_downloaded = fs::read(&version_download_path);
        let _ = fs::remove_file(&version_download_path);
        assert_eq!(version_downloaded?, version_source);

        let version_metadata = version_document.get_metadata().await?;
        assert_ne!(
            version_metadata.original_checksum,
            metadata.original_checksum
        );
        assert_eq!(
            version_metadata.original_filename.as_deref(),
            Some(version_filename.as_str())
        );
        assert_eq!(version_metadata.original_size, version_source.len() as u64);

        let version_content = support::unique_name("version-content");
        version_document.set_content(version_content.clone());
        version_document.patch().await?;
        version_document.refresh().await?;
        assert_eq!(version_document.content().as_ref(), version_content);

        let original_version = context
            .client
            .get_document_by_id(document_id, Some(original_version_id), Some(true), None)
            .await?;
        assert_eq!(
            original_version.selected_version(),
            Some(original_version_id)
        );
        assert_eq!(original_version.download_to_buffer(true).await?, source);
        assert_eq!(
            original_version.get_metadata().await?.original_checksum,
            metadata.original_checksum
        );

        let updated_label = "reviewed integration version";
        let relabeled = version_document
            .set_version_label(version_id, Some(updated_label))
            .await?;
        assert_eq!(relabeled.id, version_id);
        assert_eq!(relabeled.label.as_deref(), Some(updated_label));
        let cleared = version_document.set_version_label(version_id, None).await?;
        assert_eq!(cleared.label, None);

        let current_version_id = version_document.delete_version(version_id).await?;
        assert_eq!(current_version_id, original_version_id);
        assert_eq!(
            version_document.selected_version(),
            Some(original_version_id)
        );
        assert!(
            version_document
                .versions()
                .iter()
                .all(|version| version.id != version_id)
        );
        assert_eq!(version_document.download_to_buffer(true).await?, source);

        document.refresh().await?;
        assert!(
            document
                .versions()
                .iter()
                .all(|version| version.id != version_id)
        );
        assert_eq!(document.download_to_buffer(true).await?, source);

        context
            .client
            .acknowledge_tasks(&[task.id, second_task.id, version_task.id], false)
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
        let version_acknowledged = context
            .client
            .get_task_by_id(&version_task.id)
            .await?
            .ok_or_else(|| io::Error::other("acknowledged version task was not returned by ID"))?;
        assert!(version_acknowledged.acknowledged);

        Ok::<(), TestError>(())
    }
    .await;

    let first_cleanup = support::delete_document(&context.client, document_id).await;
    let second_cleanup = support::delete_document(&context.client, second_document_id).await;
    verification.and(first_cleanup).and(second_cleanup)?;
    assert!(
        context
            .client
            .get_document_by_id(document_id, None, None, None)
            .await
            .is_err()
    );
    assert!(
        context
            .client
            .get_document_by_id(second_document_id, None, None, None)
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
