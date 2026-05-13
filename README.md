# paperless-api &emsp; [![build]][ci] [![Latest Version]][crates.io] [![Docs]][docs.rs]

[build]: https://github.com/tripplet/paperless-api/actions/workflows/ci.yml/badge.svg
[ci]: https://github.com/tripplet/paperless-api/actions/workflows/ci.ym

[Latest Version]: https://img.shields.io/crates/v/paperless-api.svg
[crates.io]: https://crates.io/crates/paperless-api

[Docs]: https://img.shields.io/docsrs/paperless-api
[docs.rs]: https://docs.rs/paperless-api/latest/paperless_api/

A small async Rust client for interacting with the Paperless-ngx API.

This crate provides `PaperlessClient` for talking to a Paperless instance and
convenience types for working with documents, tags, custom fields,
correspondents, document types, and tasks.

## Features

- Async API built on `reqwest`
- Access documents and related metadata from Paperless
- Local metadata caching
- Upload documents
- Query task status

## Getting started

Create a client with your Paperless base URL and API token:

```rust,no_run
use paperless_api::PaperlessClient;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        None,
    )?;

    Ok(())
}
```

## Refreshing cached metadata

The client keeps some metadata cached locally, such as tags, custom fields,
correspondents, and document types.

You can refresh the caches individually or all:

```rust,no_run
use paperless_api::{PaperlessClient, RefreshMetaData};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        None,
    )?;

    client
        .refresh([
            RefreshMetaData::Tags,
            RefreshMetaData::CustomFields,
            RefreshMetaData::Correspondents,
        ])
        .await?;

    client.refresh_all().await?;

    Ok(())
}
```

## Creating and updating metadata

You can create, update and delete metadata items such as tags, correspondents,
document types, etc.
See [`CreateDtoObject`](dto::CreateDtoObject) and [`UpdateDtoObject`](dto::UpdateDtoObject)

```rust,no_run
use paperless_api::metadata::{MatchAlgorithm, tag::*};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let client = paperless_api::PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        None,
    )?;

    // Create a new tag
    let created = client.create::<Tag>(&CreateTag {
        name: "invoices".to_string(),
        color: "#ff0000".to_string(),
        ..Default::default()
    }).await?;

    // Update the tag
    client.update::<Tag>(created.id, &UpdateTag {
        name: Some("paid invoices".to_string()),
        match_pattern: Some("invoice".to_string()),
        matching_algorithm: Some(MatchAlgorithm::AnyWord),
        is_insensitive: Some(true),
        ..Default::default()
    }).await?;

    // Delete a tag
    client.delete::<Tag>(created.id).await?;

    Ok(())
}
```

## Additional headers

If your paperless instance requires additional headers to be accessed, you can provide them during client creation:

```rust,no_run
let mut headers = std::collections::HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());

let client = paperless_api::PaperlessClient::new(
    "https://paperless.example.com",
    "your-api-token",
    Some(&headers),
).expect("Error creating client");
```
