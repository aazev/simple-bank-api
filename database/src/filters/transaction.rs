use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
use uuid::Uuid;

use crate::{impl_filterable, structs::range::Range};

#[derive(Debug, Serialize, Deserialize, Default, Iterable)]
pub struct Filter {
    pub id: Option<Uuid>,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Option<Uuid>,
    pub created_at: Option<Range<NaiveDateTime>>,
    #[serde(skip_serializing, default)]
    pub offset: Option<usize>,
    #[serde(skip_serializing, default)]
    pub limit: Option<usize>,
}

impl_filterable!(
    Filter,
    exact = [id, from_account_id, to_account_id],
    range = [created_at],
    order_by = [(created_at, desc), (id, desc)]
);
