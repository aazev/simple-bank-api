use sqlx::{Postgres, Row, Transaction as SqlxTransaction};
use uuid::Uuid;

use crate::{
    filters::transaction::Filter as TransactionFilter,
    models::{
        account_dto::Account,
        transaction_dto::{Transaction, TransactionCreate},
    },
};

use super::users::UserRepository;

#[derive(Debug, Clone)]
pub struct TransactionRepository;

impl Default for TransactionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn find_all(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> anyhow::Result<Vec<Transaction>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * from transactions "#.to_owned() + &filters.query();

        let transactions = sqlx::query_as_with::<_, Transaction, _>(&query, args)
            .fetch_all(&mut **executor)
            .await?;

        Ok(transactions)
    }

    pub async fn find_one_by_filter(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> anyhow::Result<Transaction> {
        let args = filters.get_arguments();
        let query = r#"SELECT * from transactions "#.to_owned() + &filters.query();

        let transaction = sqlx::query_as_with::<_, Transaction, _>(&query, args)
            .fetch_one(&mut **executor)
            .await?;

        Ok(transaction)
    }

    pub async fn find_by_id(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        id: &Uuid,
    ) -> anyhow::Result<Transaction> {
        let transaction =
            sqlx::query_as::<_, Transaction>(r#"SELECT * from transactions WHERE id = $1"#)
                .bind(id)
                .fetch_one(&mut **executor)
                .await?;

        Ok(transaction)
    }

    pub async fn create(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        account: &Account,
        transaction_create: &TransactionCreate,
    ) -> anyhow::Result<Transaction> {
        let transaction_id = Uuid::now_v7();
        let user_repository = UserRepository::new();

        let user = user_repository
            .find_by_id(executor, &account.user_id)
            .await?;

        let transaction = transaction_create.to_transaction(&user.encryption_key)?;

        let created_transaction = sqlx::query_as::<_, Transaction>(
            r#"INSERT INTO transactions (id, operation, from_account_id, to_account_id, amount) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(transaction_id)
        .bind(&transaction.operation)
        .bind(transaction.from_account_id)
        .bind(transaction.to_account_id)
        .bind(&transaction.amount)
        .fetch_one(&mut **executor)
        .await?;

        Ok(created_transaction)
    }

    pub async fn update(
        &self,
        _executor: &mut SqlxTransaction<'_, Postgres>,
        _id: &Uuid,
        _transaction_create: &TransactionCreate,
    ) -> anyhow::Result<Transaction> {
        Err(anyhow::anyhow!("Transaction alterations are not allowed"))
    }

    pub async fn delete(&self, _executor: &mut SqlxTransaction<'_, Postgres>, _id: &Uuid) -> bool {
        false
    }

    pub async fn get_total(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> anyhow::Result<u64> {
        let args = filters.get_arguments();
        let query = r#"SELECT COUNT(*) as total FROM transactions "#.to_owned() + &filters.total();
        let result = sqlx::query_with(&query, args)
            .fetch_one(&mut **executor)
            .await?;

        Ok(result.get::<i64, &str>("total") as u64)
    }

    pub async fn delete_by_account_id(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        account_id: &Uuid,
    ) -> anyhow::Result<()> {
        sqlx::query(r#"DELETE FROM transactions WHERE to_account_id = $1"#)
            .bind(account_id)
            .execute(&mut **executor)
            .await?;

        Ok(())
    }
}
