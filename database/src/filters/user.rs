use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
use uuid::Uuid;

use crate::impl_filterable;

#[derive(Debug, Serialize, Deserialize, Default, Iterable)]
pub struct Filter {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(skip_serializing, default)]
    pub offset: Option<usize>,
    #[serde(skip_serializing, default)]
    pub limit: Option<usize>,
}

impl_filterable!(
    Filter,
    exact = [id, name, email],
    range = [],
    multi_match = [],
    order_by = [(created_at, asc), (id, asc)]
);
