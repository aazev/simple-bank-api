use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}
