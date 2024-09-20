use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Repository<IdType, ModelType, CreateType, Filter> {
    async fn find_all(&self, filters: &Filter) -> Result<Vec<ModelType>>;
    async fn find_one_by_filter(&self, filters: &Filter) -> Result<ModelType>;
    async fn find_by_id(&self, id: &IdType) -> Result<Option<ModelType>>;
    async fn create(&self, entity: &CreateType) -> Result<ModelType>;
    async fn update(&self, id: &IdType, entity: &CreateType) -> Result<ModelType>;
    async fn delete(&self, id: &IdType) -> bool;
    async fn get_total(&self, filters: &Filter) -> Result<u64>;
}
