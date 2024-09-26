use anyhow::Result;
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

#[async_trait]
pub trait Repository<IdType, ModelType, CreateType, Filter> {
    async fn find_all(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &Filter,
    ) -> Result<Vec<ModelType>>;
    async fn find_one_by_filter(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &Filter,
    ) -> Result<Option<ModelType>>;
    async fn find_by_id(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &IdType,
    ) -> Result<ModelType>;
    async fn create(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        entity: &CreateType,
    ) -> Result<ModelType>;
    async fn update(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &IdType,
        entity: &CreateType,
    ) -> Result<ModelType>;
    async fn delete(&self, executor: &mut Transaction<'_, Postgres>, id: &IdType) -> bool;
    async fn get_total(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &Filter,
    ) -> Result<u64>;
}
