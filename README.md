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

```rust
use paperless_api::PaperlessClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

You can refresh a single cache:

```rust
use paperless_api::PaperlessClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        None,
    )?;

    client.refresh_tags().await?;
    client.refresh_custom_fields().await?;

    Ok(())
}
```

You can also refresh multiple datasets at once:

```rust
use paperless_api::{PaperlessClient, RefreshData};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
```

## Working with cached metadata

After refreshing tags, you can query the local cache:

```rust
use paperless_api::PaperlessClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        None,
    )?;

    client.refresh_tags().await?;

    if let Some(tag) = client.find_tag_by_name("invoice") {
        println!("tag id: {}", tag.id.0);
    }

    Ok(())
}
```

## Additional headers

If your paperless instance requires additional headers to be accessed, you can provide them during client creation:

```rust
use std::collections::HashMap;

use paperless_api::PaperlessClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "value".to_string());

    let client = PaperlessClient::new(
        "https://paperless.example.com",
        "your-api-token",
        Some(&headers),
    )?;

    let _ = client;
    Ok(())
}
```
