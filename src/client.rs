//! The central client for interacting with Paperless.

use std::{borrow::Cow, collections::HashMap, path::Path, str::FromStr, sync::Arc};

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
    attributes::{
        correspondent::Correspondent, custom_field::CustomField, document_type::DocumentType,
        storage_path::StoragePath, tag::Tag,
    },
    document::{Document, DocumentData},
    document_query::DocumentQueryBuilder,
    dto::{CreateDto, Item, UpdateDto},
    id::{
        CorrespondentId, CustomFieldId, DocumentId, DocumentTypeId, GroupId, ItemId, StoragePathId,
        TagId, TaskId, UniqueTaskId, UserId,
    },
    task::Task,
    util,
    workflow::Workflow,
};

/// Selects which cached attributes to refresh.
///
/// Cached data is data which is rarely updated;
/// refreshing it is normally not necessary on every request.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Sequence)]
#[non_exhaustive]
pub enum RefreshAttributes {
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

/// Search hit data
#[derive(Debug, Clone, Deserialize)]
pub struct SearchHit {
    /// The score of the search hit.
    pub score: f32,

    /// Highlight of the search hit in the document content.
    pub highlights: Option<String>,

    /// Highlight of the search hit in the note content.
    pub note_highlights: Option<String>,

    /// Rank of the search hit.
    pub rank: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct SearchResult {
    #[serde(flatten)]
    document_data: DocumentData,

    #[serde(rename = "__search_hit__")]
    search_hit: SearchHit,
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

#[derive(Serialize)]
struct TokenRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum PayloadLogging {
    #[default]
    Enabled,
    Disabled,
}

impl PaperlessClient {
    /// Create a new Paperless client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Paperless API.
    /// * `token` - The authentication token for the Paperless API.
    /// * `headers` - Optional additional headers to include in requests.
    ///
    /// # Errors
    ///
    /// Returns an error if the client builder fails or headers are invalid.
    pub fn new(
        base_url: &str,
        token: &str,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<Self> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the client builder fails or headers are invalid.
    pub fn new_with_client(
        base_url: &str,
        token: &str,
        headers: Option<&HashMap<String, String>>,
        client_builder: reqwest::ClientBuilder,
    ) -> Result<Self> {
        let mut headers_map = HeaderMap::new();

        // Add additional headers if provided
        if let Some(headers) = headers {
            headers_map = create_header_map(headers)?;
        }

        // Add the Paperless token header
        headers_map.insert(
            HeaderName::from_str("Authorization")
                .map_err(|err| Error::InvalidHeader(err.to_string()))?,
            format!("Token {token}")
                .parse()
                .map_err(|err: InvalidHeaderValue| Error::InvalidHeader(err.to_string()))?,
        );

        Ok(Self {
            request_full_permissions: false,
            request_full_content: false,
            base_url: base_url.into(),
            client: client_builder
                .default_headers(headers_map)
                .build()
                .map_err(|err| Error::Other(err.to_string()))?,
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

    /// Request an API token using Paperless login credentials.
    ///
    /// The returned token can be used to create a [`PaperlessClient`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the credentials are rejected, or the response is
    /// invalid.
    pub async fn request_token(
        base_url: &str,
        headers: Option<&HashMap<String, String>>,
        client_builder: Option<reqwest::ClientBuilder>,
        username: &str,
        password: &str,
    ) -> Result<String> {
        let mut client = client_builder.unwrap_or_else(|| reqwest::Client::builder().zstd(true));

        if let Some(headers) = headers {
            client = client.default_headers(create_header_map(headers)?);
        }

        let client = client.build().map_err(|err| Error::Request(err.into()))?;

        let endpoint = format!("{}/api/token/", base_url.trim_end_matches('/'));
        let request = client
            .post(&endpoint)
            .header(ACCEPT, "application/json")
            .json(&TokenRequest { username, password })
            .build()
            .map_err(|err| Error::Request(err.into()))?;

        let response = Self::send_request(&client, request, PayloadLogging::Disabled).await?;

        Self::response_json::<TokenResponse>(response, PayloadLogging::Disabled)
            .await
            .map(|response| response.token)
    }

    /// Sets whether to request full permissions data for items during refresh.
    ///
    /// If not enabled only simple permission data is loaded.
    /// See [`ItemPermissions`](crate::attributes::permission::ItemPermissions) for more details.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn load_items<T: Item + DeserializeOwned>(&self) -> Result<HashMap<T::Id, T>> {
        let endpoint = format!("/api/{}/", T::endpoint());
        debug!(endpoint, "Loading");

        let items: Vec<T> = self.fetch_all_pages(&endpoint, None).await?;
        Ok(items.into_iter().map(|item| (item.id(), item)).collect())
    }

    fn default_query_params(&self) -> Option<HashMap<&'static str, Cow<'_, str>>> {
        let mut params = HashMap::new();

        if self.request_full_permissions {
            params.insert(
                crate::document_query::QUERY_PARAM_FULL_PERMISSIONS,
                Cow::Borrowed("true"),
            );
        }
        if !self.request_full_content {
            params.insert(
                crate::document_query::QUERY_PARAM_TRUNCATE_CONTENT,
                Cow::Borrowed("true"),
            );
        }

        if params.is_empty() {
            None
        } else {
            Some(params)
        }
    }

    /// Refresh and cache all attributes.
    ///
    /// Only updates the cache for this instance, cloned instances will not see the changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the refresh fails.
    pub async fn refresh_all(&mut self) -> Result<()> {
        self.refresh(enum_iterator::all::<RefreshAttributes>())
            .await
    }

    /// Refresh and cache the selected attributes.
    ///
    /// Only updates the cache for this instance, cloned instances will not see the changes.
    ///
    /// # Arguments
    ///
    /// * `data` - The attributes to refresh.
    /// * `full_permissions` - Whether to use request full permissions data for the items being refreshed.
    ///
    /// # Errors
    ///
    /// Returns an error if the refresh fails.
    pub async fn refresh(
        &mut self,
        data: impl IntoIterator<Item = RefreshAttributes>,
    ) -> Result<()> {
        #[rustfmt::skip]
        async fn inner(
            client: &mut PaperlessClient,
            data: &mut dyn Iterator<Item = RefreshAttributes>,
        ) -> Result<()> {
            let selected: std::collections::HashSet<_> = data.into_iter().collect();

            if selected.is_empty() {
                return Ok(());
            }

            let (tags, custom_fields, correspondents, document_types, groups, users, storage_paths) =
                futures_util::try_join!(
                    async {
                        if selected.contains(&RefreshAttributes::Tags) {
                            Ok(Some(client.load_items::<Tag>().await?))
                        } else {
                            Ok::<Option<_>, Error>(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshAttributes::CustomFields) {
                            Ok(Some(client.load_items::<CustomField>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshAttributes::Correspondents) {
                            Ok(Some(client.load_items::<Correspondent>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshAttributes::DocumentTypes) {
                            Ok(Some(client.load_items::<DocumentType>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshAttributes::Groups) {
                            Ok(Some(client.load_items::<Group>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshAttributes::Users) {
                            Ok(Some(client.load_items::<User>().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshAttributes::StoragePaths) {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn query_documents(&self, query: DocumentQueryBuilder) -> Result<Vec<Document>> {
        let full_content = query.full_content;
        let query_params = query.build();
        let query: HashMap<&str, Cow<str>> = query_params
            .query
            .into_iter()
            .map(|(k, v)| (k, Cow::Owned(v)))
            .collect();

        let doc_client = Arc::new(self.clone());
        let documents: Vec<_> = self
            .fetch_all_pages::<DocumentData>("/api/documents/", Some(&query))
            .await?
            .into_iter()
            .map(|data| Document::new(data, doc_client.clone(), !full_content))
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

    pub(crate) async fn get_document_data_by_id(
        &self,
        id: DocumentId,
        full_content: Option<bool>,
        full_permissions: Option<bool>,
    ) -> Result<DocumentData> {
        let mut params = self.default_query_params();

        if full_content.is_some() || full_permissions.is_some() {
            let mut updated_params = params.unwrap_or_default();

            if let Some(full_content) = full_content {
                updated_params.insert(
                    crate::document_query::QUERY_PARAM_TRUNCATE_CONTENT,
                    Cow::Owned((!full_content).to_string()),
                );
            }

            if let Some(full_permissions) = full_permissions {
                updated_params.insert(
                    crate::document_query::QUERY_PARAM_FULL_PERMISSIONS,
                    Cow::Owned(full_permissions.to_string()),
                );
            }

            params = Some(updated_params);
        }

        self.request_json_no_body(
            Method::GET,
            &format!("/api/documents/{}/", id.0),
            params.as_ref(),
        )
        .await
    }

    /// Get a document by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to fetch the document.
    pub async fn get_document_by_id(
        &self,
        id: DocumentId,
        full_content: Option<bool>,
        full_permissions: Option<bool>,
    ) -> Result<Document> {
        let content_is_truncated = !full_content.unwrap_or(self.request_full_content);
        Ok(Document::new(
            self.get_document_data_by_id(id, full_content, full_permissions)
                .await?,
            Arc::new(self.clone()),
            content_is_truncated,
        ))
    }

    /// Make a request and parse the response as JSON.
    pub(crate) fn request_json_no_body<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query_params: Option<&HashMap<&str, Cow<str>>>,
    ) -> impl Future<Output = Result<T>> {
        self.request_json(method, endpoint, None::<&()>, query_params)
    }

    /// Make a request and parse the response as JSON.
    pub(crate) async fn request_json<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
        query_params: Option<&HashMap<&str, Cow<'_, str>>>,
    ) -> Result<T> {
        let resp = self.request(method, endpoint, body, query_params).await?;

        Self::response_json(resp, PayloadLogging::default()).await
    }

    async fn response_json<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
        payload_logging: PayloadLogging,
    ) -> Result<T> {
        if payload_logging == PayloadLogging::Enabled && tracing::enabled!(tracing::Level::TRACE) {
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

    /// Make a request with no body and return the raw [`reqwest::Response`].
    pub(crate) fn request_no_body(
        &self,
        method: Method,
        endpoint: &str,
        query_params: Option<&HashMap<&str, Cow<'_, str>>>,
    ) -> impl Future<Output = Result<reqwest::Response>> {
        self.request(method, endpoint, None::<&()>, query_params)
    }

    /// Make a request and return the raw [`reqwest::Response`].
    pub(crate) async fn request(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
        query_params: Option<&HashMap<&str, Cow<'_, str>>>,
    ) -> Result<reqwest::Response> {
        self.request_with_logging(
            method,
            endpoint,
            body,
            query_params,
            PayloadLogging::default(),
        )
        .await
    }

    async fn request_with_logging(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&impl Serialize>,
        query_params: Option<&HashMap<&str, Cow<'_, str>>>,
        payload_logging: PayloadLogging,
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

        Self::send_request(&self.client, req, payload_logging).await
    }

    async fn send_request(
        client: &reqwest::Client,
        req: reqwest::Request,
        payload_logging: PayloadLogging,
    ) -> Result<reqwest::Response> {
        if payload_logging == PayloadLogging::Enabled
            && tracing::enabled!(tracing::Level::TRACE)
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

        let resp = client
            .execute(req)
            .await
            .map_err(|err| Error::Other(format!("Failed to send request: {err}")))?;

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
        query_params: Option<&HashMap<&str, Cow<'_, str>>>,
    ) -> Result<Vec<T>> {
        let mut results = vec![];
        let mut all_query_params = self.default_query_params().unwrap_or_default();
        if let Some(query_params) = query_params {
            all_query_params.extend(query_params.clone());
        }

        let mut all_query_params = Some(all_query_params);

        let mut current_url = Some(endpoint.to_string());

        while let Some(url) = current_url {
            debug!("Fetching page: {url}");

            let page: PaginatedResponse<T> = self
                .request_json_no_body(Method::GET, &url, all_query_params.as_ref())
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

    /// Get all tasks with optional filtering by their unique ID, name, or acknowledged status.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to fetch the task status.
    pub async fn get_task_status(
        &self,
        task_id: Option<&UniqueTaskId>,
        task_name: Option<&str>,
        acknowledged: Option<bool>,
    ) -> Result<Vec<Task>> {
        let mut query = HashMap::new();

        if let Some(id) = task_id {
            query.insert("task_id", Cow::Owned(id.to_string()));
        }

        if let Some(name) = task_name {
            query.insert("task_name", Cow::Owned(name.to_string()));
        }

        if let Some(ack) = acknowledged {
            query.insert("acknowledged", Cow::Owned(ack.to_string()));
        }

        self.fetch_all_pages("/api/tasks/", Some(&query)).await
    }

    /// Get a task by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to fetch the task.
    pub async fn get_task_by_id(&self, task_id: &crate::id::TaskId) -> Result<Option<Task>> {
        let url = format!("/api/tasks/{task_id}/");
        match self.request_json_no_body(Method::GET, &url, None).await {
            found_item @ Ok(_) => found_item,
            Err(Error::NotFound) => Ok(None),
            err @ Err(_) => err,
        }
    }

    /// Acknowledge tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A slice of [`TaskId`]s or [`Task`]s to acknowledge.
    /// * `all` - Whether to acknowledge all tasks of the same type.
    ///
    /// # Errors
    ///
    /// Paperless will reject a request if `all` is `true` and specific tasks are provided.
    pub async fn acknowledge_tasks<T>(&self, tasks: &[T], all: bool) -> Result<()>
    where
        for<'a> &'a T: Into<TaskId>,
    {
        let request = crate::task::AcknowledgeRequest {
            tasks: tasks.iter().map(std::convert::Into::into).collect(),
            all: if all { Some(true) } else { None },
        };

        self.request(
            Method::POST,
            "/api/tasks/acknowledge/",
            Some(&request),
            None,
        )
        .await?;
        Ok(())
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
    /// All structs which implement [`CreateDto`] can be used as `new_item`.
    ///
    /// Returns the created item.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to create the item on the server.
    pub async fn create<T>(&self, new_item: &T) -> Result<T::BaseType>
    where
        T: CreateDto,
        T::BaseType: Item,
    {
        let url = format!("/api/{}/", <T::BaseType as Item>::endpoint());
        self.request_json(Method::POST, &url, Some(&new_item), None)
            .await
    }

    /// Updates an existing item.
    ///
    /// All structs which implement [`UpdateDto`] can be used as `item`.
    ///
    /// Returns the updated item
    ///
    /// # Errors
    ///
    /// Returns an error if the updating of the item fails.
    pub async fn update<T>(&self, id: T::Id, update: &T) -> Result<T::BaseType>
    where
        T: UpdateDto,
        T::BaseType: Item,
    {
        let url = format!("/api/{}/{}/", <T::BaseType as Item>::endpoint(), id);
        self.request_json::<T::BaseType>(Method::PATCH, &url, Some(&update), None)
            .await
    }

    /// Deletes an existing item.
    ///
    /// Can be used for all [`ItemId`]s
    ///
    /// # Errors
    ///
    /// Returns an error if the deletion fails.
    pub async fn delete<T: ItemId>(&self, id: T) -> Result<()> {
        let url = format!("/api/{}/{}/", T::endpoint(), id);
        self.request_no_body(Method::DELETE, &url, None).await?;
        Ok(())
    }

    /// Load an existing item directly from the server, bypassing the caches.
    ///
    /// All structs which implement [`Item`] can be used.
    ///
    /// # Errors
    ///
    /// Returns an error if request fails.
    pub async fn load_by_id<T: Item>(&self, id: T::Id) -> Result<Option<T>> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the upload fail.
    pub async fn upload_document(&self, file_path: &Path, filename: &str) -> Result<UniqueTaskId> {
        let stream = tokio::fs::File::open(file_path)
            .await
            .map_err(|e| Error::Other(format!("Failed to open file: {e}")))?;

        // Send the part with a known length so that reqwest emits a Content-Length
        // header for the whole multipart body. With an unknown length it falls back
        // to `Transfer-Encoding: chunked` without Content-Length, and Django-based
        // paperless then parses the request as empty ("No file was submitted")
        // unless a buffering reverse proxy re-adds the header
        // (see https://code.djangoproject.com/ticket/35289).
        let file_len = stream
            .metadata()
            .await
            .map_err(|e| Error::Other(format!("Failed to read file metadata: {e}")))?
            .len();

        let form = multipart::Form::new().part(
            "document",
            multipart::Part::stream_with_length(stream, file_len).file_name(filename.to_string()),
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
        Ok(UniqueTaskId(task_id))
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

    /// Search for documents.
    ///
    /// # Errors
    ///
    /// Returns an error if the search request fails.
    pub async fn search(&self, search: &str) -> Result<Vec<(Document, SearchHit)>> {
        let doc_client = Arc::new(self.clone());

        let results = self
            .fetch_all_pages::<SearchResult>(
                "/api/documents/",
                Some(&HashMap::from([("query", search.into())])),
            )
            .await?
            .into_iter()
            .map(|result| {
                (
                    Document::new(
                        result.document_data,
                        doc_client.clone(),
                        !self.request_full_content,
                    ),
                    result.search_hit,
                )
            })
            .collect();
        Ok(results)
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

fn create_header_map(headers: &HashMap<String, String>) -> Result<HeaderMap> {
    let mut headers_map = HeaderMap::new();

    for (key, value) in headers {
        headers_map.insert(
            HeaderName::from_str(key).map_err(|err| Error::InvalidHeader(err.to_string()))?,
            value
                .parse()
                .map_err(|err: InvalidHeaderValue| Error::InvalidHeader(err.to_string()))?,
        );
    }

    Ok(headers_map)
}
