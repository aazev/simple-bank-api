use serde::Serialize;
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
