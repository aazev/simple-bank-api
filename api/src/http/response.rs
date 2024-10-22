use axum::{http::StatusCode, Json};
use serde::{Serialize, Serializer};
use utoipa::ToSchema;

use super::validation::ValidationField;

#[derive(Debug, Serialize, ToSchema)]
pub struct HttpResponse {
    pub status: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<ValidationField>>,
}

impl HttpResponse {
    pub fn new(status: u16, message: String, fields: Option<Vec<ValidationField>>) -> Self {
        Self {
            status,
            message,
            fields,
        }
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            status: 200,
            message: "Ok".to_string(),
            fields: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HttpPaginatedResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<T>>,
    pub total: u64,
    pub offset: usize,
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<usize>,
}

impl<T: Serialize> HttpPaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: usize, limit: Option<usize>, total: u64) -> Self {
        let next_page = match total > (page * limit.unwrap_or(25)).try_into().unwrap() {
            true => Some(page + 1),
            false => None,
        };

        Self {
            data: Some(data),
            total,
            offset: page,
            limit: limit.unwrap_or(25),
            next_page,
        }
    }
}

impl<T: Serialize> Default for HttpPaginatedResponse<T> {
    fn default() -> Self {
        Self {
            data: None,
            total: 0,
            offset: 1,
            limit: 25,
            next_page: None,
        }
    }
}

#[derive(Debug, ToSchema)]
pub enum ReturnTypes<T> {
    Paginated(HttpPaginatedResponse<T>),
    Multiple(Vec<T>),
    Single(T),
}

impl<T: Serialize> Serialize for ReturnTypes<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ReturnTypes::Paginated(paginated) => paginated.serialize(serializer),
            ReturnTypes::Multiple(multiple) => multiple.serialize(serializer),
            ReturnTypes::Single(single) => single.serialize(serializer),
        }
    }
}

#[allow(dead_code)]
pub fn validate_pagination(
    offset: Option<usize>,
    limit: Option<usize>,
) -> anyhow::Result<(), (StatusCode, Json<String>)> {
    match (offset, limit) {
        (Some(_), None) => Err((
            StatusCode::BAD_REQUEST,
            Json("LIMIT is required when OFFSET is present".to_string()),
        )),
        (None, Some(_)) => Err((
            StatusCode::BAD_REQUEST,
            Json("OFFSET is required when LIMIT is present".to_string()),
        )),
        _ => Ok(()),
    }
}
