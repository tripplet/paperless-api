//! Utility types.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Statistics {
    /// Total number of documents.
    pub documents_total: u32,

    /// Number of documents in the inbox.
    pub documents_inbox: u32,

    /// Tag used for documents in the inbox.
    pub inbox_tag: u32,

    /// Tags used for documents in the inbox.
    pub inbox_tags: Vec<u32>,

    /// Counts of document file types.
    pub document_file_type_counts: Vec<DocumentFileTypeCount>,

    /// Total number of characters in all documents.
    pub character_count: u64,

    /// Total number of tags.
    pub tag_count: u32,

    /// Total number of correspondents.
    pub correspondent_count: u32,

    /// Total number of document types.
    pub document_type_count: u32,

    /// Total number of storage paths.
    pub storage_path_count: u32,

    /// Current ASN.
    pub current_asn: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentFileTypeCount {
    /// MIME type of the document type.
    pub mime_type: String,

    /// Number of documents of this type.
    #[serde(rename = "mime_type_count")]
    pub count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerStatus {
    /// The version of Paperless-ngx.
    #[serde(rename = "pngx_version")]
    pub version: String,

    /// Operating system of the server.
    pub server_os: String,

    /// Type of installation (e.g. docker).
    pub install_type: String,

    /// Storage information.
    pub storage: Storage,

    /// Database information.
    pub database: Database,

    /// Task status information.
    pub tasks: StatusTask,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Storage {
    pub total: u64,
    pub available: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawDatabase")]
pub struct Database {
    pub db_type: String,
    pub url: String,
    pub status: Health,
    pub migration_status: MigrationStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MigrationStatus {
    pub latest_migration: String,
    pub unapplied_migrations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Health {
    #[serde(rename = "OK")]
    Ok,

    #[serde(untagged)]
    Error(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusTask {
    #[serde(flatten)]
    pub redis: RedisStatus,

    #[serde(flatten)]
    pub celery: CeleryStatus,

    #[serde(flatten)]
    pub index: IndexStatus,

    #[serde(flatten)]
    pub sanity_check: SanityCheckStatus,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawRedisStatus")]
pub struct RedisStatus {
    pub url: String,
    pub status: Health,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawCeleryStatus")]
pub struct CeleryStatus {
    pub status: Health,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawIndexStatus")]
pub struct IndexStatus {
    pub status: Health,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawClassifierStatus")]
pub struct ClassifierStatus {
    pub status: Health,
    pub last_trained: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawSanityCheckStatus")]
pub struct SanityCheckStatus {
    pub status: Health,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize)]
pub struct RawDatabase {
    #[serde(rename = "type")]
    pub db_type: String,
    pub url: String,
    pub status: String,
    pub error: Option<String>,
    pub migration_status: MigrationStatus,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct RawRedisStatus {
    redis_url: String,
    redis_status: String,
    redis_error: Option<String>,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct RawCeleryStatus {
    celery_status: String,
    celery_url: String,
    celery_error: Option<String>,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct RawIndexStatus {
    index_status: String,
    index_last_modified: Option<chrono::DateTime<chrono::Utc>>,
    index_error: Option<String>,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct RawClassifierStatus {
    classifier_status: String,
    classifier_last_trained: Option<chrono::DateTime<chrono::Utc>>,
    classifier_error: Option<String>,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct RawSanityCheckStatus {
    sanity_check_status: String,
    sanity_check_last_run: Option<chrono::DateTime<chrono::Utc>>,
    sanity_check_error: Option<String>,
}

impl From<RawDatabase> for Database {
    fn from(raw: RawDatabase) -> Self {
        Self {
            db_type: raw.db_type,
            url: raw.url,
            status: merge_status_with_error(raw.status, raw.error),
            migration_status: raw.migration_status,
        }
    }
}

impl From<RawRedisStatus> for RedisStatus {
    fn from(raw: RawRedisStatus) -> Self {
        Self {
            url: raw.redis_url,
            status: merge_status_with_error(raw.redis_status, raw.redis_error),
        }
    }
}

impl From<RawCeleryStatus> for CeleryStatus {
    fn from(raw: RawCeleryStatus) -> Self {
        Self {
            status: merge_status_with_error(raw.celery_status, raw.celery_error),
            url: raw.celery_url,
        }
    }
}

impl From<RawIndexStatus> for IndexStatus {
    fn from(raw: RawIndexStatus) -> Self {
        Self {
            status: merge_status_with_error(raw.index_status, raw.index_error),
            last_modified: raw.index_last_modified,
        }
    }
}

impl From<RawClassifierStatus> for ClassifierStatus {
    fn from(raw: RawClassifierStatus) -> Self {
        Self {
            status: merge_status_with_error(raw.classifier_status, raw.classifier_error),
            last_trained: raw.classifier_last_trained,
        }
    }
}

impl From<RawSanityCheckStatus> for SanityCheckStatus {
    fn from(raw: RawSanityCheckStatus) -> Self {
        Self {
            status: merge_status_with_error(raw.sanity_check_status, raw.sanity_check_error),
            last_run: raw.sanity_check_last_run,
        }
    }
}

impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Health::Ok => write!(f, "OK"),
            Health::Error(err) => write!(f, "Error: {err}"),
        }
    }
}

fn merge_status_with_error(status: String, error: Option<String>) -> Health {
    if let Some(error) = error {
        Health::Error(error)
    } else if status.to_lowercase() != "ok" {
        Health::Error(status)
    } else {
        Health::Ok
    }
}

impl ServerStatus {
    /// Returns `Health::Ok` if all components report `Ok`.
    /// Otherwise returns `Health::Error` with a combined message of all failures.
    #[must_use]
    pub fn overall(&self) -> Health {
        let mut errors = Vec::new();

        if let Health::Error(ref err) = self.database.status {
            errors.push(format!("database: {err}"));
        }
        if let Health::Error(ref err) = self.tasks.redis.status {
            errors.push(format!("task redis: {err}"));
        }
        if let Health::Error(ref err) = self.tasks.celery.status {
            errors.push(format!("task celery: {err}"));
        }
        if let Health::Error(ref err) = self.tasks.index.status {
            errors.push(format!("task index: {err}"));
        }
        if let Health::Error(ref err) = self.tasks.sanity_check.status {
            errors.push(format!("task sanity_check: {err}"));
        }

        if errors.is_empty() {
            Health::Ok
        } else {
            Health::Error(errors.join(", "))
        }
    }
}
