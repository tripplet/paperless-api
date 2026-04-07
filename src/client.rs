use std::{collections::HashMap, path::Path, str::FromStr, sync::Arc};

use reqwest::{
    Method, StatusCode,
    header::{ACCEPT, HeaderMap, HeaderName, InvalidHeaderValue},
    multipart,
};
use serde::Deserialize;
use tracing::{debug, trace};

use crate::{
    Error, Result, User,
    correspondent::Correspondent,
    custom_field::CustomField,
    document::{Document, DocumentData},
    document_type::DocumentType,
    id::{CorrespondentId, CustomFieldId, DocumentId, DocumentTypeId, TagId, TaskId, UserId},
    tag::Tag,
    task::Task,
    workflow::Workflow,
};

/// Selects which cached metadata to refresh.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RefreshData {
    Tags,
    CustomFields,
    Correspondents,
    DocumentTypes,
    Users,
}

/// Client to interact with Paperless.
#[derive(Debug, Clone)]
pub struct PaperlessClient {
    client: reqwest::Client,
    pub(crate) base_url: String,

    correspondents: HashMap<CorrespondentId, Correspondent>,
    document_types: HashMap<DocumentTypeId, DocumentType>,
    tags: HashMap<TagId, Tag>,
    custom_fields: HashMap<CustomFieldId, CustomField>,
    users: HashMap<UserId, User>,
}

#[derive(Debug, Deserialize)]
struct PaginatedResponse<T> {
    results: Vec<T>,
    next: Option<String>,
}

impl PaperlessClient {
    /// Create a new Paperless client.
    pub fn new(
        base_url: &str,
        token: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> std::result::Result<Self, String> {
        let mut headers_map = HeaderMap::new();

        // Add additional headers if provided
        if let Some(headers) = headers {
            for (key, value) in headers {
                headers_map.insert(
                    HeaderName::from_str(key).map_err(|err| err.to_string())?,
                    value
                        .parse()
                        .map_err(|err: InvalidHeaderValue| err.to_string())?,
                );
            }
        }

        // Add the Paperless token header
        headers_map.insert(
            HeaderName::from_str("Authorization").map_err(|err| err.to_string())?,
            format!("Token {token}")
                .parse()
                .map_err(|err: InvalidHeaderValue| err.to_string())?,
        );

        Ok(Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::builder()
                .default_headers(headers_map)
                .zstd(true)
                .build()
                .map_err(|err| err.to_string())?,
            tags: HashMap::new(),
            custom_fields: HashMap::new(),
            correspondents: HashMap::new(),
            document_types: HashMap::new(),
            users: HashMap::new(),
        })
    }

    async fn load_tags(&self) -> Result<HashMap<TagId, Tag>> {
        debug!("loading tags");
        let tags: Vec<Tag> = self.fetch_all_pages("/api/tags/").await?;
        Ok(tags.into_iter().map(|tag| (tag.id, tag)).collect())
    }

    async fn load_custom_fields(&self) -> Result<HashMap<CustomFieldId, CustomField>> {
        debug!("loading custom fields");
        let custom_fields: Vec<CustomField> = self.fetch_all_pages("/api/custom_fields/").await?;
        Ok(custom_fields
            .into_iter()
            .map(|custom_field| (custom_field.id, custom_field))
            .collect())
    }

    async fn load_correspondents(&self) -> Result<HashMap<CorrespondentId, Correspondent>> {
        debug!("loading correspondents");
        let correspondents: Vec<Correspondent> =
            self.fetch_all_pages("/api/correspondents/").await?;
        Ok(correspondents
            .into_iter()
            .map(|correspondent| (correspondent.id, correspondent))
            .collect())
    }

    async fn load_document_types(&self) -> Result<HashMap<DocumentTypeId, DocumentType>> {
        debug!("loading document types");
        let document_types: Vec<DocumentType> =
            self.fetch_all_pages("/api/document_types/").await?;
        Ok(document_types
            .into_iter()
            .map(|document_type| (document_type.id, document_type))
            .collect())
    }

    async fn load_users(&self) -> Result<HashMap<UserId, User>> {
        debug!("loading users");
        let users: Vec<User> = self.fetch_all_pages("/api/users/").await?;
        Ok(users.into_iter().map(|user| (user.id, user)).collect())
    }

    pub async fn refresh_all(&mut self) -> Result<()> {
        self.refresh([
            RefreshData::Tags,
            RefreshData::CustomFields,
            RefreshData::Correspondents,
            RefreshData::DocumentTypes,
            RefreshData::Users,
        ])
        .await
    }

    /// Refresh selected cached metadata concurrently.
    pub async fn refresh(&mut self, data: impl IntoIterator<Item = RefreshData>) -> Result<()> {
        let mut refresh_tags = false;
        let mut refresh_custom_fields = false;
        let mut refresh_correspondents = false;
        let mut refresh_document_types = false;
        let mut refresh_users = false;

        for item in data {
            match item {
                RefreshData::Tags => refresh_tags = true,
                RefreshData::CustomFields => refresh_custom_fields = true,
                RefreshData::Correspondents => refresh_correspondents = true,
                RefreshData::DocumentTypes => refresh_document_types = true,
                RefreshData::Users => refresh_users = true,
            }
        }

        let (tags, custom_fields, correspondents, document_types, users) = futures_util::try_join!(
            async {
                if refresh_tags {
                    Ok::<Option<HashMap<TagId, Tag>>, Error>(Some(self.load_tags().await?))
                } else {
                    Ok::<Option<HashMap<TagId, Tag>>, Error>(None)
                }
            },
            async {
                if refresh_custom_fields {
                    Ok::<Option<HashMap<CustomFieldId, CustomField>>, Error>(Some(
                        self.load_custom_fields().await?,
                    ))
                } else {
                    Ok::<Option<HashMap<CustomFieldId, CustomField>>, Error>(None)
                }
            },
            async {
                if refresh_correspondents {
                    Ok::<Option<HashMap<CorrespondentId, Correspondent>>, Error>(Some(
                        self.load_correspondents().await?,
                    ))
                } else {
                    Ok::<Option<HashMap<CorrespondentId, Correspondent>>, Error>(None)
                }
            },
            async {
                if refresh_document_types {
                    Ok::<Option<HashMap<DocumentTypeId, DocumentType>>, Error>(Some(
                        self.load_document_types().await?,
                    ))
                } else {
                    Ok::<Option<HashMap<DocumentTypeId, DocumentType>>, Error>(None)
                }
            },
            async {
                if refresh_users {
                    Ok::<Option<HashMap<UserId, User>>, Error>(Some(self.load_users().await?))
                } else {
                    Ok::<Option<HashMap<UserId, User>>, Error>(None)
                }
            },
        )?;

        if let Some(tags) = tags {
            self.tags = tags;
        }

        if let Some(custom_fields) = custom_fields {
            self.custom_fields = custom_fields;
        }

        if let Some(correspondents) = correspondents {
            self.correspondents = correspondents;
        }

        if let Some(document_types) = document_types {
            self.document_types = document_types;
        }

        if let Some(users) = users {
            self.users = users;
        }

        Ok(())
    }

    /// Refresh tags.
    #[inline]
    pub async fn refresh_tags(&mut self) -> Result<()> {
        self.refresh([RefreshData::Tags]).await
    }

    /// Refresh custom fields.
    #[inline]
    pub async fn refresh_custom_fields(&mut self) -> Result<()> {
        self.refresh([RefreshData::CustomFields]).await
    }

    /// Refresh correspondents.
    #[inline]
    pub async fn refresh_correspondents(&mut self) -> Result<()> {
        self.refresh([RefreshData::Correspondents]).await
    }

    /// Refresh document types.
    #[inline]
    pub async fn refresh_document_types(&mut self) -> Result<()> {
        self.refresh([RefreshData::DocumentTypes]).await
    }

    /// Refresh users.
    #[inline]
    pub async fn refresh_users(&mut self) -> Result<()> {
        self.refresh([RefreshData::Users]).await
    }

    /// Get all documents with any of the given tags.
    pub async fn get_documents_by_tags(
        &self,
        tag_ids: &[TagId],
        truncate_content: bool,
    ) -> Result<Vec<Document>> {
        let tag_id_str = tag_ids
            .iter()
            .map(|tag_id| tag_id.0.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let documents: Vec<_> = self
            .fetch_all_pages::<DocumentData>(&format!(
                "/api/documents/?truncate_content={truncate_content}&tags__id__in={tag_id_str}"
            ))
            .await?
            .into_iter()
            .map(|data| Document::new(data, Arc::new(self.clone()), truncate_content))
            .collect();

        Ok(documents)
    }

    pub(crate) async fn get_document_data_by_id(&self, id: DocumentId) -> Result<DocumentData> {
        let resp = self
            .request(Method::GET, &format!("/api/documents/{}/", id.0), None)
            .await?;

        let document_data: DocumentData = resp
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to parse document: {e}")))?;

        Ok(document_data)
    }

    /// Get a document by its ID.
    pub async fn get_document_by_id(&self, id: DocumentId) -> Result<Document> {
        Ok(Document::new(
            self.get_document_data_by_id(id).await?,
            Arc::new(self.clone()),
            false,
        ))
    }

    pub(crate) async fn request(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let mut req = self
            .client
            .request(method, format!("{}{endpoint}", self.base_url))
            .header(ACCEPT, "application/json");

        // Set payload body if provided
        if let Some(json_body) = body {
            req = req.json(json_body);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| Error::Other(format!("Failed to send request: {e}")))?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Err(Error::NotFound);
        }

        if !resp.status().is_success() {
            return Err(Error::Response {
                status_code: resp.status().as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        Ok(resp)
    }

    pub(crate) async fn fetch_all_pages<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
    ) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut current_url = Some(endpoint.to_string());

        while let Some(url) = current_url {
            let resp = self.request(Method::GET, &url, None).await?;

            let page: PaginatedResponse<T> = resp.json().await.map_err(|e| {
                Error::InvalidJson(format!(
                    "Failed to parse paginated response for {endpoint}: {e}"
                ))
            })?;

            results.extend(page.results);

            current_url = page.next.and_then(|next_url| {
                // Extract just the path from the full URL
                next_url
                    .trim_start_matches(&self.base_url)
                    .to_string()
                    .into()
            });
        }

        Ok(results)
    }

    /// Get all tasks with optional filtering by ID, name, or acknowledged status.
    pub async fn get_task_status(
        &self,
        task_id: Option<&TaskId>,
        task_name: Option<&str>,
        acknowledged: Option<bool>,
    ) -> Result<Vec<Task>> {
        let mut query = Vec::new();

        if let Some(id) = task_id {
            query.push(("task_id", id.to_string()));
        }

        if let Some(name) = task_name {
            query.push(("task_name", name.to_string()));
        }

        if let Some(ack) = acknowledged {
            query.push(("acknowledged", ack.to_string()));
        }

        let resp = self
            .request(
                Method::GET,
                &format!(
                    "/api/tasks/?{}",
                    serde_urlencoded::to_string(&query)
                        .map_err(|e| Error::Other(format!("Failed to serialize query: {e}")))?
                ),
                None,
            )
            .await?;

        trace!("get_task_status response: {:?}", resp);

        let body = resp
            .text()
            .await
            .map_err(|e| Error::Other(format!("Failed to read response body: {e}")))?;

        let tasks: Vec<Task> = match serde_json::from_str(&body) {
            Ok(t) => t,
            Err(e) => {
                return Err(Error::InvalidJson(format!(
                    "Failed to parse response body: {e}"
                )));
            }
        };

        if tasks.is_empty() {
            return Err(Error::NotFound);
        }

        Ok(tasks)
    }

    pub async fn get_workflows(&self) -> Result<Vec<Workflow>> {
        self.fetch_all_pages("/api/workflows/").await
    }

    /// Upload a document to Paperless.
    ///
    /// Returns the task ID on success.
    pub async fn upload_document(&self, file_path: &Path, filename: &str) -> Result<TaskId> {
        let file_bytes = std::fs::read(file_path)
            .map_err(|e| Error::Other(format!("Failed to read file: {e}")))?;

        let form = multipart::Form::new().part(
            "document",
            multipart::Part::bytes(file_bytes).file_name(filename.to_string()),
        );

        let url = format!("{}/api/documents/post_document/", self.base_url);

        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::Other(format!("Failed to send request: {e}")))?;

        let status = resp.status();
        if !resp.status().is_success() {
            return Err(Error::Response {
                status_code: status.as_u16(),
                body: resp.text().await.unwrap_or_default(),
            });
        }

        let task_id: String = resp
            .json()
            .await
            .map_err(|e| Error::Other(format!("Failed to parse task ID: {e}")))?;
        Ok(TaskId(task_id))
    }

    #[inline]
    #[must_use]
    pub fn tags(&self) -> &HashMap<TagId, Tag> {
        &self.tags
    }

    #[must_use]
    pub fn find_tag_by_name(&self, name: &str) -> Option<&Tag> {
        self.tags.values().find(|tag| tag.name == name)
    }

    #[inline]
    #[must_use]
    pub fn document_types(&self) -> &HashMap<DocumentTypeId, DocumentType> {
        &self.document_types
    }

    #[must_use]
    pub fn find_document_type_by_name(&self, name: &str) -> Option<&DocumentType> {
        self.document_types.values().find(|dt| dt.name == name)
    }

    #[inline]
    #[must_use]
    pub fn correspondents(&self) -> &HashMap<CorrespondentId, Correspondent> {
        &self.correspondents
    }

    #[inline]
    #[must_use]
    pub fn custom_fields(&self) -> &HashMap<CustomFieldId, CustomField> {
        &self.custom_fields
    }

    #[must_use]
    pub fn find_custom_field_by_name(&self, name: &str) -> Option<&CustomField> {
        self.custom_fields.values().find(|field| field.name == name)
    }

    #[inline]
    #[must_use]
    pub fn users(&self) -> &HashMap<UserId, User> {
        &self.users
    }
}
