//! Query for documents using the [`DocumentQueryBuilder`].

use crate::{
    document::ArchiveSerialNumber,
    id::{CorrespondentId, TagId},
};

/// Builder for constructing document queries.
#[derive(Default)]
pub struct DocumentQueryBuilder {
    archive_serial_number: Option<ArchiveSerialNumber>,
    correspondent_id_in: Option<Vec<CorrespondentId>>,
    correspondent_name_icontains: Option<String>,
    content_icontains: Option<String>,
    tags_id_in: Option<Vec<TagId>>,
    pub(crate) full_content: bool,
    full_permissions: bool,
}

/// A constructed document query.
pub struct DocumentQuery {
    pub(crate) query: Vec<(&'static str, String)>,
}

pub(crate) const QUERY_PARAM_FULL_PERMISSIONS: &str = "full_perms";
pub(crate) const QUERY_PARAM_TRUNCATE_CONTENT: &str = "truncate_content";
const QUERY_PARAM_TAGS_ID_IN: &str = "tags__id__in";
const QUERY_PARAM_ARCHIVE_SERIAL_NUMBER: &str = "archive_serial_number";
const QUERY_PARAM_CORRESPONDENT_ID_IN: &str = "correspondent__id__in";
const QUERY_PARAM_CORRESPONDENT_NAME_ICONTAINS: &str = "correspondent__name__icontains";
const QUERY_PARAM_CONTENT_ICONTAINS: &str = "content__icontains";

impl DocumentQueryBuilder {
    /// Filters documents which have the given archive serial number.
    #[must_use]
    pub fn archive_serial_number(mut self, archive_serial_number: ArchiveSerialNumber) -> Self {
        self.archive_serial_number = Some(archive_serial_number);
        self
    }

    /// Filters documents which have any of the given correspondents.
    #[must_use]
    pub fn correspondent_id_in(mut self, correspondent_id_in: Vec<CorrespondentId>) -> Self {
        self.correspondent_id_in = Some(correspondent_id_in);
        self
    }

    /// Filters documents which have a correspondent name containing the given string.
    #[must_use]
    pub fn correspondent_name_icontains(mut self, correspondent_name_icontains: String) -> Self {
        self.correspondent_name_icontains = Some(correspondent_name_icontains);
        self
    }

    /// Filters documents which have content containing the given string.
    #[must_use]
    pub fn content_icontains(mut self, content_icontains: String) -> Self {
        self.content_icontains = Some(content_icontains);
        self
    }

    /// Filters documents which have any of the given tags.
    #[must_use]
    pub fn tags_id_in(mut self, tags_id_in: Vec<TagId>) -> Self {
        self.tags_id_in = Some(tags_id_in);
        self
    }

    /// Returns documents with full content (truncated by default to save bandwidth).
    #[must_use]
    pub fn full_content(mut self, full_content: bool) -> Self {
        self.full_content = full_content;
        self
    }

    /// Returns documents with full permissions data.
    #[must_use]
    pub fn full_permissions(mut self, full_permissions: bool) -> Self {
        self.full_permissions = full_permissions;
        self
    }

    /// Builds the query.
    #[must_use]
    pub fn build(self) -> DocumentQuery {
        let mut query = vec![];

        if let Some(archive_serial_number) = self.archive_serial_number {
            query.push((
                QUERY_PARAM_ARCHIVE_SERIAL_NUMBER,
                archive_serial_number.0.to_string(),
            ));
        }
        if let Some(correspondent_id_in) = self.correspondent_id_in {
            query.push((
                QUERY_PARAM_CORRESPONDENT_ID_IN,
                correspondent_id_in
                    .iter()
                    .map(|id| id.0.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if let Some(correspondent_name_icontains) = self.correspondent_name_icontains {
            query.push((
                QUERY_PARAM_CORRESPONDENT_NAME_ICONTAINS,
                correspondent_name_icontains,
            ));
        }
        if let Some(content_icontains) = self.content_icontains {
            query.push((QUERY_PARAM_CONTENT_ICONTAINS, content_icontains));
        }
        if let Some(tags_id_in) = self.tags_id_in {
            query.push((
                QUERY_PARAM_TAGS_ID_IN,
                tags_id_in
                    .iter()
                    .map(|id| id.0.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if !self.full_content {
            query.push((QUERY_PARAM_TRUNCATE_CONTENT, "true".to_string()));
        }

        if self.full_permissions {
            query.push((QUERY_PARAM_FULL_PERMISSIONS, "true".to_string()));
        }

        DocumentQuery { query }
    }
}
