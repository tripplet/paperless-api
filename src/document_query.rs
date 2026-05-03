//! Query for Documents using the [`DocumentQueryBuilder`]

use crate::{
    document::ArchiveSerialNumber,
    id::{CorrespondentId, TagId},
};

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

pub struct DocumentQuery {
    pub(crate) query: Vec<(&'static str, String)>,
}

const QUERY_PARAM_FULL_PERMISSIONS: &str = "full_perms";
const QUERY_PARAM_TRUNCATE_CONTENT: &str = "truncate_content";
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
    pub fn full_permissions(mut self) -> Self {
        self.full_permissions = true;
        self
    }

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
            query.push((QUERY_PARAM_TRUNCATE_CONTENT, "false".to_string()));
        }

        if self.full_content {
            query.push((QUERY_PARAM_FULL_PERMISSIONS, "true".to_string()));
        }

        DocumentQuery { query }
    }
}
/*
added__date__gt
string($date)
(query)

added__date__gte
string($date)
(query)

added__date__lt
string($date)
(query)

added__date__lte
string($date)
(query)

added__day
number
(query)

added__gt
string($date-time)
(query)

added__gte
string($date-time)
(query)

added__lt
string($date-time)
(query)

added__lte
string($date-time)
(query)

added__month
number
(query)

added__year
number
(query)

content__icontains
string
(query)

content__iendswith
string
(query)

content__iexact
string
(query)

content__istartswith
string
(query)

correspondent__id
integer
(query)

correspondent__id__in
array<integer>
(query)
Mehrere Werte können durch Kommas getrennt sein.
Add integer item
correspondent__id__none
integer
(query)

correspondent__isnull
boolean
(query)

correspondent__name__icontains
string
(query)

correspondent__name__iendswith
string
(query)

correspondent__name__iexact
string
(query)

correspondent__name__istartswith
string
(query)

created__date__gt
string($date)
(query)

created__date__gte
string($date)
(query)

created__date__lt
string($date)
(query)

created__date__lte
string($date)
(query)

created__day
number
(query)

created__gt
string($date)
(query)

created__gte
string($date)
(query)

created__lt
string($date)
(query)

created__lte
string($date)
(query)

created__month
number
(query)

created__year
number
(query)

custom_field_query
string
(query)

custom_fields__icontains
string
(query)

custom_fields__id__all
integer
(query)

custom_fields__id__in
integer
(query)

custom_fields__id__none
integer
(query)

document_type__id
integer
(query)

document_type__id__in
array<integer>
(query)
Mehrere Werte können durch Kommas getrennt sein.
Add integer item
document_type__id__none
integer
(query)

document_type__isnull
boolean
(query)

document_type__name__icontains
string
(query)

document_type__name__iendswith
string
(query)

document_type__name__iexact
string
(query)

document_type__name__istartswith
string
(query)

fields
array<string>
(query)
Add string item
full_perms
boolean
(query)

has_custom_fields
boolean
(query)
Has custom field

id
integer
(query)

id__in
array<integer>
(query)
Mehrere Werte können durch Kommas getrennt sein.
Add integer item
is_in_inbox
boolean
(query)

is_tagged
boolean
(query)
Is tagged

mime_type
string
(query)

modified__date__gt
string($date)
(query)

modified__date__gte
string($date)
(query)

modified__date__lt
string($date)
(query)

modified__date__lte
string($date)
(query)

modified__day
number
(query)

modified__gt
string($date-time)
(query)

modified__gte
string($date-time)
(query)

modified__lt
string($date-time)
(query)

modified__lte
string($date-time)
(query)

modified__month
number
(query)

modified__year
number
(query)

ordering
string
(query)
Feld, das zum Sortieren der Ergebnisse verwendet werden soll.

original_filename__icontains
string
(query)

original_filename__iendswith
string
(query)

original_filename__iexact
string
(query)

original_filename__istartswith
string
(query)

owner__id
integer
(query)

owner__id__in
array<integer>
(query)
Mehrere Werte können durch Kommas getrennt sein.
Add integer item
owner__id__none
integer
(query)

owner__isnull
boolean
(query)

page
integer
(query)
Eine Seitenzahl in der paginierten Ergebnismenge.

page_size
integer
(query)
Anzahl der pro Seite zurückzugebenden Ergebnisse.

query
string
(query)
Advanced search query string

search
string
(query)
Ein Suchbegriff.

shared_by__id
boolean
(query)

storage_path__id
integer
(query)

storage_path__id__in
array<integer>
(query)
Mehrere Werte können durch Kommas getrennt sein.
Add integer item
storage_path__id__none
integer
(query)

storage_path__isnull
boolean
(query)

storage_path__name__icontains
string
(query)

storage_path__name__iendswith
string
(query)

storage_path__name__iexact
string
(query)

storage_path__name__istartswith
string
(query)

tags__id
integer
(query)

tags__id__all
integer
(query)

tags__id__in
integer
(query)

tags__id__none
integer
(query)

tags__name__icontains
string
(query)

tags__name__iendswith
string
(query)

tags__name__iexact
string
(query)

tags__name__istartswith
string
(query)

title__icontains
string
(query)

title__iendswith
string
(query)

title__iexact
string
(query)

title__istartswith
string
(query)

title_content
string
(query)
 */
