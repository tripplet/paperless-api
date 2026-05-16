//! The central client for interacting with Paperless.

use std::{collections::HashMap, path::Path, str::FromStr, sync::Arc};

use enum_iterator::Sequence;
use reqwest::{
    Method, StatusCode,
    header::{ACCEPT, HeaderMap, HeaderName, InvalidHeaderValue},
    multipart,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{debug, trace};

use crate::{
    Error, Group, Result, SavedView, User,
    document::{Document, DocumentData},
    document_query::DocumentQueryBuilder,
    dto::{CreateDto, Item, UpdateDto},
    id::{
        CorrespondentId, CustomFieldId, DocumentId, DocumentTypeId, GroupId, StoragePathId, TagId,
        TaskId, UserId,
    },
    metadata::{
        correspondent::Correspondent, custom_field::CustomField, document_type::DocumentType,
        storage_path::StoragePath, tag::Tag,
    },
    task::Task,
    util,
    workflow::Workflow,
};

/// Selects which cached metadata to refresh.
///
/// Cached data is data which is rarely updated;
/// refreshing it is normally not necessary on every request.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Sequence)]
#[non_exhaustive]
pub enum RefreshMetaData {
    Tags,
    CustomFields,
    Correspondents,
    DocumentTypes,
    Groups,
    Users,
    StoragePaths,
}

/// Client to interact with Paperless.
#[derive(Debug, Clone)]
pub struct PaperlessClient {
    /// Whether to request full permissions data for items.
    pub request_full_permissions: bool,

    /// Whether to always request the full document content.
    pub request_full_content: bool,

    pub(crate) base_url: Arc<str>,

    client: reqwest::Client,
    cached_data: Arc<CachedData>,
}

#[derive(Debug, Clone)]
struct CachedData {
    correspondents: HashMap<CorrespondentId, Correspondent>,
    custom_fields: HashMap<CustomFieldId, CustomField>,
    document_types: HashMap<DocumentTypeId, DocumentType>,
    groups: HashMap<GroupId, Group>,
    storage_paths: HashMap<StoragePathId, StoragePath>,
    tags: HashMap<TagId, Tag>,
    users: HashMap<UserId, User>,
}

#[derive(Debug, Deserialize)]
struct PaginatedResponse<T> {
    results: Vec<T>,
    next: Option<String>,
}

impl PaperlessClient {
    /// Create a new Paperless client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Paperless API.
    /// * `token` - The authentication token for the Paperless API.
    /// * `headers` - Optional additional headers to include in requests.
    pub fn new(
        base_url: &str,
        token: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> std::result::Result<Self, String> {
        Self::new_with_client(
            base_url,
            token,
            headers,
            reqwest::Client::builder().zstd(true),
        )
    }

    /// Create a new Paperless client.
    ///
    /// Provide a [`reqwest::ClientBuilder`] to customize the HTTP client,
    /// such as adding custom headers or disabling compression.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Paperless API.
    /// * `token` - The authentication token for the Paperless API.
    /// * `headers` - Optional additional headers to include in requests.
    /// * `client_builder` - [`reqwest::ClientBuilder`] to use for creating the HTTP client.
    pub fn new_with_client(
        base_url: &str,
        token: &str,
        headers: Option<&HashMap<String, String>>,
        client_builder: reqwest::ClientBuilder,
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
            request_full_permissions: false,
            request_full_content: false,
            base_url: base_url.into(),
            client: client_builder
                .default_headers(headers_map)
                .build()
                .map_err(|err| err.to_string())?,
            cached_data: Arc::new(CachedData {
                custom_fields: HashMap::new(),
                correspondents: HashMap::new(),
                document_types: HashMap::new(),
                groups: HashMap::new(),
                storage_paths: HashMap::new(),
                tags: HashMap::new(),
                users: HashMap::new(),
            }),
        })
    }

    /// Sets whether to request full permissions data for items during refresh.
    ///
    /// If not enabled only simple permission data is loaded.
    /// See [`ItemPermissions`](crate::metadata::permission::ItemPermissions) for more details.
    #[must_use]
    pub fn with_full_permissions(mut self, req: bool) -> Self {
        self.request_full_permissions = req;
        self
    }

    #[must_use]
    pub fn with_full_content(mut self, full_content: bool) -> Self {
        self.request_full_content = full_content;
        self
    }

    /// Loads all items of the given item type from the API.
    pub async fn load_items<T: Item + DeserializeOwned>(&self) -> Result<HashMap<T::Id, T>> {
        debug!("Loading {}", T::endpoint());
        let endpoint = format!("/api/{}/", T::endpoint());

        let items: Vec<T> = self.fetch_all_pages(&endpoint, None).await?;
        Ok(items.into_iter().map(|item| (item.id(), item)).collect())
    }

    fn default_query_params(&self) -> Option<Vec<(&'static str, &'static str)>> {
        let mut params = Vec::with_capacity(2);

        if self.request_full_permissions {
            params.push((crate::document_query::QUERY_PARAM_FULL_PERMISSIONS, "true"));
        }
        if !self.request_full_content {
            params.push((crate::document_query::QUERY_PARAM_TRUNCATE_CONTENT, "true"));
        }

        if params.is_empty() {
            None
        } else {
            Some(params)
        }
    }

    /// Refresh and cache all metadata.
    ///
    /// Only updates the cache for this instance, cloned instances will not see the changes.
    pub async fn refresh_all(&mut self) -> Result<()> {
        self.refresh(enum_iterator::all::<RefreshMetaData>()).await
    }

    /// Refresh and cache the selected metadata.
    ///
    /// Only updates the cache for this instance, cloned instances will not see the changes.
    ///
    /// # Arguments
    ///
    /// * `data` - The metadata to refresh.
    /// * `full_permissions` - Whether to use request full permissions data for the items being refreshed.
    pub async fn refresh(&mut self, data: impl IntoIterator<Item = RefreshMetaData>) -> Result<()> {
        #[rustfmt::skip]
        async fn inner(
            client: &mut PaperlessClient,
            data: &mut dyn Iterator<Item = RefreshMetaData>,
        ) -> Result<()> {
            let selected: std::collections::HashSet<_> = data.into_iter().collect();

            if selected.is_empty() {
                return Ok(());
            }

            let (tags, custom_fields, correspondents, document_types, groups, users, storage_paths) =
                futures_util::try_join!(
                    async {
                        if selected.contains(&RefreshMetaData::Tags) {
                            Ok(Some(client.load_items::<Tag>().await?))
                        } else {
                            Ok::<Option<_>, Error>(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::CustomFields) {
                            Ok(Some(client.load_items::<CustomField>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::Correspondents) {
                            Ok(Some(client.load_items::<Correspondent>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::DocumentTypes) {
                            Ok(Some(client.load_items::<DocumentType>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::Groups) {
                            Ok(Some(client.load_items::<Group>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::Users) {
                            Ok(Some(client.load_items::<User>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::StoragePaths) {
                            Ok(Some(client.load_items::<StoragePath>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                )?;

            let cached_data = Arc::make_mut(&mut client.cached_data);

            if let Some(value) = custom_fields { cached_data.custom_fields = value; }
            if let Some(value) = correspondents { cached_data.correspondents = value; }
            if let Some(value) = document_types { cached_data.document_types = value; }
            if let Some(value) = groups { cached_data.groups = value; }
            if let Some(value) = storage_paths { cached_data.storage_paths = value; }
            if let Some(value) = tags { cached_data.tags = value; }
            if let Some(value) = users { cached_data.users = value; }

            Ok(())
        }

        inner(self, &mut data.into_iter()).await
    }

    /// Query documents using the given [`DocumentQueryBuilder`].
    pub async fn query_documents(&self, query: DocumentQueryBuilder) -> Result<Vec<Document>> {
        let full_content = query.full_content;
        let query_params = query.build();
        let query_vec: Vec<_> = query_params
            .query
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();
        let query_slice = query_vec.as_slice();

        let documents: Vec<_> = self
            .fetch_all_pages::<DocumentData>("/api/documents/", Some(query_slice))
            .await?
            .into_iter()
            .map(|data| Document::new(data, Arc::new(self.clone()), !full_content))
            .collect();

        Ok(documents)
    }

    /// Get all documents with any of the given tags.
    pub fn get_documents_by_tags(
        &self,
        tag_ids: &[TagId],
    ) -> impl Future<Output = Result<Vec<Document>>> {
        let query = DocumentQueryBuilder::default()
            .full_content(self.request_full_content)
            .full_permissions(self.request_full_permissions)
            .tags_id_in(tag_ids.to_vec());

        self.query_documents(query)
    }

    pub(crate) async fn get_document_data_by_id(&self, id: DocumentId) -> Result<DocumentData> {
        self.request_json_no_body(
            Method::GET,
            &format!("/api/documents/{}/", id.0),
            self.default_query_params().as_deref(),
        )
        .await
    }

    /// Get a document by its ID.
    pub async fn get_document_by_id(&self, id: DocumentId) -> Result<Document> {
        Ok(Document::new(
            self.get_document_data_by_id(id).await?,
            Arc::new(self.clone()),
            false,
        ))
    }

    /// Make a request and parse the response as JSON.
    pub(crate) fn request_json_no_body<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query_params: Option<&[(&str, &str)]>,
    ) -> impl Future<Output = Result<T>> {
        self.request_json(method, endpoint, None::<&()>, query_params)
    }

    /// Make a request and parse the response as JSON.
    pub(crate) async fn request_json<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
        query_params: Option<&[(&str, &str)]>,
    ) -> Result<T> {
        let resp = self.request(method, endpoint, body, query_params).await?;

        if tracing::enabled!(tracing::Level::TRACE) {
            // Only log the response body if trace logging is enabled to avoid unnecessary overhead
            let response_text = resp.text().await.unwrap_or_default();
            trace!(body = %response_text, "Response");

            Ok(serde_json::from_str(&response_text)
                .map_err(|e| Error::InvalidJson(format!("Failed to parse response body: {e:?}")))?)
        } else {
            Ok(resp
                .json()
                .await
                .map_err(|e| Error::InvalidJson(format!("Failed to parse response body: {e:?}")))?)
        }
    }

    /// Make a request and return the raw [`reqwest::Response`].
    pub(crate) fn request_no_body(
        &self,
        method: Method,
        endpoint: &str,
        query_params: Option<&[(&str, &str)]>,
    ) -> impl Future<Output = Result<reqwest::Response>> {
        self.request(method, endpoint, None::<&()>, query_params)
    }

    /// Make a request and return the raw [`reqwest::Response`].
    pub(crate) async fn request(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
        query_params: Option<&[(&str, &str)]>,
    ) -> Result<reqwest::Response> {
        let mut req = self
            .client
            .request(method, format!("{}{endpoint}", self.base_url))
            .header(ACCEPT, "application/json");

        if let Some(params) = query_params {
            req = req.query(params);
        }

        // Set payload body if provided
        if let Some(json_body) = body {
            req = req.json(json_body);
        }

        let req = req.build().map_err(|e| Error::Request(e.into()))?;

        if tracing::enabled!(tracing::Level::TRACE)
            && let Some(body) = req.body().and_then(|b| b.as_bytes())
        {
            trace!(
                method = ?req.method(),
                url = ?req.url(),
                body = %String::from_utf8_lossy(body),
                "Sending request to Paperless API");
        } else {
            debug!(
                method = ?req.method(),
                url = ?req.url(),
                "Sending request to Paperless API");
        }

        let resp = self
            .client
            .execute(req)
            .await
            .map_err(|e| Error::Other(format!("Failed to send request: {e}")))?;

        // Log the response body for debugging
        debug!(status = ?resp.status(), "Response");

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
        query_params: Option<&[(&str, &str)]>,
    ) -> Result<Vec<T>> {
        let mut results = vec![];
        let mut all_query_params = self.default_query_params().unwrap_or_default();
        all_query_params.extend(query_params.unwrap_or_default());
        let mut all_query_params = Some(all_query_params);

        let mut current_url = Some(endpoint.to_string());

        while let Some(url) = current_url {
            debug!("Fetching page: {url}");

            let page: PaginatedResponse<T> = self
                .request_json_no_body(Method::GET, &url, all_query_params.as_deref())
                .await?;

            results.extend(page.results);

            current_url = page.next.and_then(|next_url| {
                // Extract just the path from the full URL
                next_url
                    .strip_prefix(&*self.base_url)
                    .unwrap_or(&next_url)
                    .to_string()
                    .into()
            });
            all_query_params = None;
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
            .request_no_body(
                Method::GET,
                &format!(
                    "/api/tasks/?{}",
                    serde_urlencoded::to_string(&query)
                        .map_err(|e| Error::Other(format!("Failed to serialize query: {e}")))?
                ),
                None,
            )
            .await?;

        let body = resp
            .text()
            .await
            .map_err(|e| Error::Other(format!("Failed to read response body: {e:?}")))?;

        trace!("get_task_status response: {:?}", body);

        let tasks: Vec<Task> = match serde_json::from_str(&body) {
            Ok(t) => t,
            Err(e) => {
                return Err(Error::InvalidJson(format!(
                    "Failed to parse response body: {e:?}"
                )));
            }
        };

        if tasks.is_empty() {
            return Err(Error::NotFound);
        }

        Ok(tasks)
    }

    /// Get all workflows.
    pub fn get_workflows(&self) -> impl Future<Output = Result<Vec<Workflow>>> {
        self.fetch_all_pages("/api/workflows/", None)
    }

    /// Get all saved views.
    pub fn get_saved_views(&self) -> impl Future<Output = Result<Vec<SavedView>>> {
        self.fetch_all_pages("/api/saved_views/", None)
    }

    /// Get server statistics.
    pub fn get_statistics(&self) -> impl Future<Output = Result<util::Statistics>> {
        self.request_json_no_body(Method::GET, "/api/statistics/", None)
    }

    /// Get server status.
    pub fn get_status(&self) -> impl Future<Output = Result<util::ServerStatus>> {
        self.request_json_no_body(Method::GET, "/api/status/", None)
    }

    /// Create a new item on the server.
    ///
    /// All structs which implement [`CreateDtoObject`](crate::dto::CreateDtoObject) can be used as `new_item`.
    ///
    /// Returns the created item.
    pub async fn create<T: CreateDto>(&self, new_item: &T) -> Result<T::BaseType> {
        let url = format!("/api/{}/", T::endpoint());
        self.request_json(Method::POST, &url, Some(&new_item), None)
            .await
    }

    /// Updates an existing.
    ///
    /// All structs which implement [`UpdateDtoObject`](crate::dto::UpdateDtoObject) can be used as `item`.
    ///
    /// Returns the updated item
    pub async fn update<T: UpdateDto>(&self, id: T::Id, update: &T) -> Result<T::BaseType> {
        let url = format!("/api/{}/{}/", T::endpoint(), id);
        self.request_json::<T::BaseType>(Method::PATCH, &url, Some(&update), None)
            .await
    }

    /// Deletes an existing item.
    ///
    /// All structs which implement [`UpdateDtoObject`](crate::dto::UpdateDtoObject) can be used.
    pub async fn delete<T: Item>(&self, id: T::Id) -> Result<()> {
        let url = format!("/api/{}/{}/", T::endpoint(), id);
        self.request_no_body(Method::DELETE, &url, None).await?;
        Ok(())
    }

    /// Load an existing item directly from the server, bypassing the caches.
    ///
    /// All structs which implement [`Item`] can be used.
    pub async fn load_by_id<T: Item>(&self, id: T::Id) -> Result<Option<T::BaseType>> {
        let url = format!("/api/{}/{}/", T::endpoint(), id);
        match self.request_json_no_body(Method::GET, &url, None).await {
            found_item @ Ok(_) => found_item,
            Err(Error::NotFound) => Ok(None),
            err @ Err(_) => err,
        }
    }

    /// Upload a document to Paperless.
    ///
    /// Returns the task ID on success.
    pub async fn upload_document(&self, file_path: &Path, filename: &str) -> Result<TaskId> {
        let stream = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| Error::Other(format!("Failed to open file: {e}")))?;

        let form = multipart::Form::new().part(
            "document",
            multipart::Part::stream(stream).file_name(filename.to_string()),
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
            .map_err(|e| Error::Other(format!("Failed to parse task ID: {e:?}")))?;
        Ok(TaskId(task_id))
    }

    /// Get the tags cache.
    #[inline]
    #[must_use]
    pub fn tags(&self) -> &HashMap<TagId, Tag> {
        &self.cached_data.tags
    }

    /// Get the storage paths cache.
    #[inline]
    #[must_use]
    pub fn storage_paths(&self) -> &HashMap<StoragePathId, StoragePath> {
        &self.cached_data.storage_paths
    }

    /// Find a tag by its name.
    #[must_use]
    pub fn find_tag_by_name(&self, name: &str) -> Option<&Tag> {
        self.cached_data.tags.values().find(|tag| tag.name == name)
    }

    /// Get the document types cache.
    #[inline]
    #[must_use]
    pub fn document_types(&self) -> &HashMap<DocumentTypeId, DocumentType> {
        &self.cached_data.document_types
    }

    /// Find a document type by its name.
    #[must_use]
    pub fn find_document_type_by_name(&self, name: &str) -> Option<&DocumentType> {
        self.cached_data
            .document_types
            .values()
            .find(|dt| dt.name == name)
    }

    /// Get the correspondents cache.
    #[inline]
    #[must_use]
    pub fn correspondents(&self) -> &HashMap<CorrespondentId, Correspondent> {
        &self.cached_data.correspondents
    }

    /// Get the custom fields cache.
    #[inline]
    #[must_use]
    pub fn custom_fields(&self) -> &HashMap<CustomFieldId, CustomField> {
        &self.cached_data.custom_fields
    }

    /// Find a custom field by its name.
    #[must_use]
    pub fn find_custom_field_by_name(&self, name: &str) -> Option<&CustomField> {
        self.cached_data
            .custom_fields
            .values()
            .find(|field| field.name == name)
    }

    /// Get the users cache.
    #[inline]
    #[must_use]
    pub fn users(&self) -> &HashMap<UserId, User> {
        &self.cached_data.users
    }

    /// Get the groups cache.
    #[inline]
    #[must_use]
    pub fn groups(&self) -> &HashMap<GroupId, Group> {
        &self.cached_data.groups
    }
}
