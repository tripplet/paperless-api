# paperless-api

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
use paperless_api::{PaperlessClient, RefreshData};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        None,
    )?;

    client
        .refresh([
            RefreshData::Tags,
            RefreshData::CustomFields,
            RefreshData::Correspondents,
        ])
        .await?;
    
    client.refresh_all().await?;

    Ok(())
}
```

## Additional headers

If your paperless instance requires additional headers to be accessed, you can provide them during client creation:

```rust,no_run
use std::collections::HashMap;
use paperless_api::PaperlessClient;

let mut headers = HashMap::new();
headers.insert("X-Custom-Header".to_string(), "value".to_string());

let client = PaperlessClient::new(
    "https://paperless.example.com",
    "your-api-token",
    Some(&headers),
).expect("Error creating client");
```
