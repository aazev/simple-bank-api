use async_trait::async_trait;
use sqlx::{Postgres, Row, Transaction as SqlxTransaction};
use uuid::Uuid;

use crate::{
    filters::transaction::Filter as TransactionFilter,
    models::transaction_dto::{Transaction, TransactionCreate},
    traits::repository::Repository,
};

use super::{accounts::AccountRepository, users::UserRepository};

#[derive(Debug, Clone)]
pub struct TransactionRepository;
impl TransactionRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Repository<Uuid, Transaction, TransactionCreate, TransactionFilter> for TransactionRepository {
    async fn find_all(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> anyhow::Result<Vec<Transaction>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * from users "#.to_owned() + &filters.query();

        let transactions = sqlx::query_as_with::<_, Transaction, _>(&query, args)
            .fetch_all(*executor)
            .await?;

        Ok(transactions)
    }

    async fn find_one_by_filter(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> anyhow::Result<Transaction> {
        let args = filters.get_arguments();
        let query = r#"SELECT * from users "#.to_owned() + &filters.query();

        let transaction = sqlx::query_as_with::<_, Transaction, _>(&query, args)
            .fetch_one(*executor)
            .await?;

        Ok(transaction)
    }

    async fn find_by_id(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        id: &Uuid,
    ) -> anyhow::Result<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(r#"SELECT * from users WHERE id = $1"#)
            .bind(id)
            .fetch_one(*executor)
            .await?;

        Ok(transaction)
    }

    async fn create(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        transaction_create: &TransactionCreate,
    ) -> anyhow::Result<Transaction> {
        let account_repository = AccountRepository::new(self.db_pool.clone());
        let user_repository = UserRepository::new(self.db_pool.clone());

        let account = account_repository
            .find_by_id(&transaction_create.to_account_id)
            .await?;
        let user = user_repository.find_by_id(&account.user_id).await?;

        let transaction = transaction_create.to_transaction(&user.encryption_key)?;

        let created_transaction = sqlx::query_as::<_, Transaction>(
            r#"INSERT INTO transactions (type, from_account_id, to_account_id, amount) VALUES ($1, $2, $3, $4) RETURNING *"#,
        )
        .bind(&transaction._type)
        .bind(&transaction.from_account_id)
        .bind(&transaction.to_account_id)
        .bind(&transaction.amount)
        .fetch_one(*executor)
        .await?;

        Ok(created_transaction)
    }

    async fn update(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        _id: &Uuid,
        _transaction_create: &TransactionCreate,
    ) -> anyhow::Result<Transaction> {
        Err(anyhow::anyhow!("Transaction alterations are not allowed"))
    }

    async fn delete(&self, executor: &mut SqlxTransaction<'_, Postgres>, _id: &Uuid) -> bool {
        false
    }

    async fn get_total(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> anyhow::Result<u64> {
        let args = filters.get_arguments();
        let query = r#"SELECT COUNT(*) as total FROM transactions "#.to_owned() + &filters.total();
        let result = sqlx::query_with(&query, args).fetch_one(*executor).await?;

        Ok(result.get::<i64, &str>("total") as u64)
    }
}
