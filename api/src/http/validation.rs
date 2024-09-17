use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ValidationField {
    pub field: String,
    pub message: String,
}
