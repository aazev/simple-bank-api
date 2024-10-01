use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
use uuid::Uuid;

use crate::impl_filterable;

#[derive(Debug, Serialize, Deserialize, Default, Iterable)]
pub struct Filter {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub bank_id: Option<i32>,
    pub bank_account_number: Option<i32>,
    pub bank_agency_number: Option<i32>,
    #[serde(skip_serializing, default)]
    pub offset: Option<usize>,
    #[serde(skip_serializing, default)]
    pub limit: Option<usize>,
}

impl_filterable!(
    Filter,
    exact = [
        id,
        user_id,
        bank_id,
        bank_account_number,
        bank_agency_number
    ],
    range = [],
    order_by = [(created_at, asc), (id, asc)]
);
