use std::io;

use paperless_api::{
    RefreshAttributes,
    attributes::{
        Correspondent, DocumentType, MatchAlgorithm, Tag,
        correspondent::{CreateCorrespondent, UpdateCorrespondent},
        document_type::{CreateDocumentType, UpdateDocumentType},
        tag::{CreateTag, UpdateTag},
    },
};

use crate::support::{self, TestContext, TestError, TestResult};

pub(crate) async fn attribute_lifecycles(context: &TestContext) -> TestResult {
    tag_lifecycle(context).await?;
    correspondent_lifecycle(context).await?;
    document_type_lifecycle(context).await?;
    Ok(())
}

async fn tag_lifecycle(context: &TestContext) -> TestResult {
    let name = support::unique_name("tag");
    let updated_name = format!("{name}-updated");
    let created = context
        .client
        .create(&CreateTag {
            name,
            color: "#3366cc".to_owned(),
            match_pattern: "integration tag".to_owned(),
            matching_algorithm: MatchAlgorithm::AllWords,
            is_insensitive: true,
            ..Default::default()
        })
        .await?;

    let verification = async {
        let updated = context
            .client
            .update(
                created.id,
                &UpdateTag {
                    name: Some(updated_name.clone()),
                    color: Some("#cc6633".to_owned()),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(updated.name, updated_name);

        let loaded = context
            .client
            .load_by_id::<Tag>(created.id)
            .await?
            .ok_or_else(|| io::Error::other("created tag was not returned by ID"))?;
        assert_eq!(loaded.name, updated_name);

        let mut cached_client = context.client.clone();
        cached_client.refresh([RefreshAttributes::Tags]).await?;
        assert_eq!(
            cached_client
                .find_tag_by_name(&updated_name)
                .map(|tag| tag.id),
            Some(created.id)
        );

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
            .is_none()
    );
    Ok(())
}

async fn correspondent_lifecycle(context: &TestContext) -> TestResult {
    let name = support::unique_name("correspondent");
    let updated_name = format!("{name}-updated");
    let created = context
        .client
        .create(&CreateCorrespondent {
            name,
            match_pattern: "integration correspondent".to_owned(),
            matching_algorithm: MatchAlgorithm::AllWords,
            is_insensitive: true,
            ..Default::default()
        })
        .await?;

    let verification = async {
        let updated = context
            .client
            .update(
                created.id,
                &UpdateCorrespondent {
                    name: Some(updated_name.clone()),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(updated.name, updated_name);

        let loaded = context
            .client
            .load_by_id::<Correspondent>(created.id)
            .await?
            .ok_or_else(|| io::Error::other("created correspondent was not returned by ID"))?;
        assert_eq!(loaded.name, updated_name);

        let mut cached_client = context.client.clone();
        cached_client
            .refresh([RefreshAttributes::Correspondents])
            .await?;
        assert_eq!(
            cached_client
                .correspondents()
                .get(&created.id)
                .map(|item| item.name.as_str()),
            Some(updated_name.as_str())
        );

        Ok::<(), TestError>(())
    }
    .await;

    let cleanup = context.client.delete(created.id).await.map_err(Into::into);
    verification.and(cleanup)?;
    assert!(
        context
            .client
            .load_by_id::<Correspondent>(created.id)
            .await?
            .is_none()
    );
    Ok(())
}

async fn document_type_lifecycle(context: &TestContext) -> TestResult {
    let name = support::unique_name("document-type");
    let updated_name = format!("{name}-updated");
    let created = context
        .client
        .create(&CreateDocumentType {
            name,
            match_pattern: "integration document type".to_owned(),
            matching_algorithm: MatchAlgorithm::AllWords,
            is_insensitive: Some(true),
            ..Default::default()
        })
        .await?;

    let verification = async {
        let updated = context
            .client
            .update(
                created.id,
                &UpdateDocumentType {
                    name: Some(updated_name.clone()),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(updated.name, updated_name);

        let loaded = context
            .client
            .load_by_id::<DocumentType>(created.id)
            .await?
            .ok_or_else(|| io::Error::other("created document type was not returned by ID"))?;
        assert_eq!(loaded.name, updated_name);

        let mut cached_client = context.client.clone();
        cached_client
            .refresh([RefreshAttributes::DocumentTypes])
            .await?;
        assert_eq!(
            cached_client
                .find_document_type_by_name(&updated_name)
                .map(|item| item.id),
            Some(created.id)
        );

        Ok::<(), TestError>(())
    }
    .await;

    let cleanup = context.client.delete(created.id).await.map_err(Into::into);
    verification.and(cleanup)?;
    assert!(
        context
            .client
            .load_by_id::<DocumentType>(created.id)
            .await?
            .is_none()
    );
    Ok(())
}

#[tokio::test]
#[ignore = "debug helper; run explicitly with --ignored"]
async fn debug_attribute_lifecycles() -> TestResult {
    attribute_lifecycles(support::context().await?).await
}
