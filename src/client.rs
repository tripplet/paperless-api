//! The central client for interacting with Paperless.

use std::{collections::HashMap, fmt::Write, path::Path, str::FromStr, sync::Arc};

use enum_iterator::Sequence;
use reqwest::{
    Method, StatusCode,
    header::{ACCEPT, HeaderMap, HeaderName, InvalidHeaderValue},
    multipart,
};
use serde::Deserialize;
use tracing::{debug, trace};

use crate::{
    Error, Group, Result, User,
    correspondent::Correspondent,
    custom_field::CustomField,
    document::{Document, DocumentData},
    document_type::DocumentType,
    id::{
        CorrespondentId, CustomFieldId, DocumentId, DocumentTypeId, GroupId, StoragePathId, TagId,
        TaskId, UserId,
    },
    storage_path::StoragePath,
    tag::Tag,
    task::Task,
    workflow::Workflow,
};

const QUERY_PARAM_FULL_PERMISSIONS: &str = "full_perms";
const QUERY_PARAM_TRUNCATE_CONTENT: &str = "truncate_content";
const QUERY_PARAM_TAGS_ID_IN: &str = "tags__id__in";

/// Selects which cached metadata to refresh.
///
/// Cached data is data which is rarly updated,
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

    pub(crate) base_url: Box<str>,

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
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .default_headers(headers_map)
                .zstd(true)
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

    /// Sets whether to request full permissions data for items.
    #[must_use]
    pub fn request_full_permissions(mut self, req: bool) -> Self {
        self.request_full_permissions = req;
        self
    }

    async fn load_tags(&self) -> Result<HashMap<TagId, Tag>> {
        debug!("loading tags");
        let tags: Vec<Tag> = self
            .fetch_all_pages("/api/tags/", self.permissions_query_param())
            .await?;
        Ok(tags.into_iter().map(|tag| (tag.id, tag)).collect())
    }

    async fn load_custom_fields(&self) -> Result<HashMap<CustomFieldId, CustomField>> {
        debug!("loading custom fields");
        let custom_fields: Vec<CustomField> =
            self.fetch_all_pages("/api/custom_fields/", None).await?;
        Ok(custom_fields
            .into_iter()
            .map(|custom_field| (custom_field.id, custom_field))
            .collect())
    }

    async fn load_correspondents(&self) -> Result<HashMap<CorrespondentId, Correspondent>> {
        debug!("loading correspondents");
        let correspondents: Vec<Correspondent> = self
            .fetch_all_pages("/api/correspondents/", self.permissions_query_param())
            .await?;
        Ok(correspondents
            .into_iter()
            .map(|correspondent| (correspondent.id, correspondent))
            .collect())
    }

    async fn load_document_types(&self) -> Result<HashMap<DocumentTypeId, DocumentType>> {
        debug!("loading document types");
        let document_types: Vec<DocumentType> = self
            .fetch_all_pages("/api/document_types/", self.permissions_query_param())
            .await?;
        Ok(document_types
            .into_iter()
            .map(|document_type| (document_type.id, document_type))
            .collect())
    }

    async fn load_groups(&self) -> Result<HashMap<GroupId, Group>> {
        debug!("loading groups");
        let groups: Vec<Group> = self.fetch_all_pages("/api/groups/", None).await?;
        Ok(groups.into_iter().map(|group| (group.id, group)).collect())
    }

    async fn load_users(&self) -> Result<HashMap<UserId, User>> {
        debug!("loading users");
        let users: Vec<User> = self.fetch_all_pages("/api/users/", None).await?;
        Ok(users.into_iter().map(|user| (user.id, user)).collect())
    }

    async fn load_storage_paths(&self) -> Result<HashMap<StoragePathId, StoragePath>> {
        debug!("loading storage paths");
        let storage_paths: Vec<StoragePath> = self
            .fetch_all_pages("/api/storage_paths/", self.permissions_query_param())
            .await?;
        Ok(storage_paths
            .into_iter()
            .map(|storage_path| (storage_path.id, storage_path))
            .collect())
    }

    fn permissions_query_param(&self) -> Option<&'static [(&'static str, &'static str)]> {
        if self.request_full_permissions {
            Some(&[(QUERY_PARAM_FULL_PERMISSIONS, "true")])
        } else {
            None
        }
    }

    /// Refresh and cache all metadata.
    ///
    /// Only updates the cache for this instance, cloned instances will not see the changes.
    ///
    /// # Arguments
    ///
    /// * `full_permissions` - Whether to use request full permissions data for the items.
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
                            Ok(Some(client.load_tags().await?))
                        } else {
                            Ok::<Option<_>, Error>(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::CustomFields) {
                            Ok(Some(client.load_custom_fields().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::Correspondents) {
                            Ok(Some(client.load_correspondents().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::DocumentTypes) {
                            Ok(Some(client.load_document_types().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::Groups) {
                            Ok(Some(client.load_groups().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::Users) {
                            Ok(Some(client.load_users().await?))
                        } else {
                            Ok(None)
                        }
                    },
                    async {
                        if selected.contains(&RefreshMetaData::StoragePaths) {
                            Ok(Some(client.load_storage_paths().await?))
                        } else {
                            Ok(None)
                        }
                    },
                )?;

            let cached_data = Arc::make_mut(&mut client.cached_data);

            if let Some(value) = correspondents { cached_data.correspondents = value; }
            if let Some(value) = document_types { cached_data.document_types = value; }
            if let Some(value) = groups { cached_data.groups = value; }
            if let Some(value) = tags { cached_data.tags = value; }
            if let Some(value) = custom_fields { cached_data.custom_fields = value; }
            if let Some(value) = users { cached_data.users = value; }
            if let Some(value) = storage_paths { cached_data.storage_paths = value; }

            Ok(())
        }

        inner(self, &mut data.into_iter()).await
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
            .fetch_all_pages::<DocumentData>(
                "/api/documents/",
                Some(&[
                    (QUERY_PARAM_TAGS_ID_IN, &tag_id_str),
                    (QUERY_PARAM_TRUNCATE_CONTENT, &format!("{truncate_content}")),
                ]),
            )
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
        query_params: Option<&[(&str, &str)]>,
    ) -> Result<Vec<T>> {
        let mut results = Vec::new();
        let mut current_url = endpoint.to_string();
        let mut first_param = true;

        if let Some(params) = query_params {
            for param in params {
                if first_param {
                    current_url.push('?');
                    first_param = false;
                } else {
                    current_url.push('&');
                }
                let _ = write!(current_url, "{}={}", param.0, param.1);
            }
        }

        let mut current_url = Some(current_url);

        while let Some(url) = current_url {
            let resp = self.request(Method::GET, &url, None).await?;

            let page: PaginatedResponse<T> = resp.json().await.map_err(|e| {
                Error::InvalidJson(format!(
                    "Failed to parse paginated response for {endpoint}: {e:?}"
                ))
            })?;

            results.extend(page.results);

            current_url = page.next.and_then(|next_url| {
                // Extract just the path from the full URL
                next_url
                    .trim_start_matches(&*self.base_url)
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

    pub fn get_workflows(&self) -> impl Future<Output = Result<Vec<Workflow>>> {
        self.fetch_all_pages("/api/workflows/", None)
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
        &self.cached_data.tags
    }

    #[inline]
    #[must_use]
    pub fn storage_paths(&self) -> &HashMap<StoragePathId, StoragePath> {
        &self.cached_data.storage_paths
    }

    #[must_use]
    pub fn find_tag_by_name(&self, name: &str) -> Option<&Tag> {
        self.cached_data.tags.values().find(|tag| tag.name == name)
    }

    #[inline]
    #[must_use]
    pub fn document_types(&self) -> &HashMap<DocumentTypeId, DocumentType> {
        &self.cached_data.document_types
    }

    #[must_use]
    pub fn find_document_type_by_name(&self, name: &str) -> Option<&DocumentType> {
        self.cached_data
            .document_types
            .values()
            .find(|dt| dt.name == name)
    }

    #[inline]
    #[must_use]
    pub fn correspondents(&self) -> &HashMap<CorrespondentId, Correspondent> {
        &self.cached_data.correspondents
    }

    #[inline]
    #[must_use]
    pub fn custom_fields(&self) -> &HashMap<CustomFieldId, CustomField> {
        &self.cached_data.custom_fields
    }

    #[must_use]
    pub fn find_custom_field_by_name(&self, name: &str) -> Option<&CustomField> {
        self.cached_data
            .custom_fields
            .values()
            .find(|field| field.name == name)
    }

    #[inline]
    #[must_use]
    pub fn users(&self) -> &HashMap<UserId, User> {
        &self.cached_data.users
    }

    #[inline]
    #[must_use]
    pub fn groups(&self) -> &HashMap<GroupId, Group> {
        &self.cached_data.groups
    }
}
