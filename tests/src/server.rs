use paperless_api::{RefreshAttributes, attributes::Tag, util::Health};

use crate::support::{self, TestContext, TestResult};

pub(crate) async fn read_only_endpoints(context: &TestContext) -> TestResult {
    let status = context.client.get_status().await?;
    assert!(!status.version.is_empty());
    assert_eq!(status.install_type, "docker");
    assert!(matches!(status.database.status, Health::Ok));
    assert!(matches!(status.tasks.redis.status, Health::Ok));
    assert!(matches!(status.tasks.celery.status, Health::Ok));
    assert!(status.storage.available <= status.storage.total);
    assert!(
        status
            .database
            .migration_status
            .unapplied_migrations
            .is_empty()
    );

    let statistics = context.client.get_statistics().await?;
    let tags = context.client.load_items::<Tag>().await?;
    assert_eq!(statistics.tag_count as usize, tags.len());

    context.client.get_workflows().await?;
    context.client.get_saved_views().await?;

    let mut cached_client = context.client.clone();
    cached_client
        .refresh([
            RefreshAttributes::Tags,
            RefreshAttributes::Correspondents,
            RefreshAttributes::DocumentTypes,
            RefreshAttributes::Users,
            RefreshAttributes::Groups,
            RefreshAttributes::StoragePaths,
            RefreshAttributes::CustomFields,
        ])
        .await?;
    assert!(!cached_client.users().is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "debug helper; run explicitly with --ignored"]
async fn debug_read_only_endpoints() -> TestResult {
    read_only_endpoints(support::context().await?).await
}
