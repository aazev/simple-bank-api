use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ValidationField {
    pub field: String,
    pub message: String,
}
