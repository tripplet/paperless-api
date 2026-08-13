#[cfg(test)]
mod attributes;
#[cfg(test)]
mod documents;
#[cfg(test)]
mod server;
#[cfg(test)]
mod support;

#[cfg(test)]
mod tests {
    use crate::{attributes, documents, server, support};

    #[tokio::test]
    async fn ordered_integration_suite() -> support::TestResult {
        let context = support::context().await?;

        // Rust does not order separate test cases, so CI runs stateful scenarios here.
        documents::document_lifecycle(context).await?;
        attributes::attribute_lifecycles(context).await?;
        server::read_only_endpoints(context).await?;

        Ok(())
    }
}
