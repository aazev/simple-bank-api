use sqlx::{PgPool, Postgres, Transaction as SqlxTransaction};
use uuid::Uuid;

use crate::{
    filters::transaction::Filter as TransactionFilter,
    models::transaction_dto::{Transaction, TransactionCreate},
    repositories::{accounts::AccountRepository, transactions::TransactionRepository},
};

#[derive(Debug)]
pub struct Service {
    account_repository: AccountRepository,
    transaction_repository: TransactionRepository,
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Service {
    pub fn new() -> Self {
        Self {
            account_repository: AccountRepository::new(),
            transaction_repository: TransactionRepository::new(),
        }
    }

    pub async fn get_one_by_id(&self, db_pool: &PgPool, id: &Uuid) -> Option<Transaction> {
        // if we had a logging system, we would log the error here
        (self.transaction_repository.find_by_id(db_pool, id).await).ok()
    }

    pub async fn get_all(
        &self,
        db_pool: &PgPool,
        filters: &TransactionFilter,
    ) -> (Vec<Transaction>, u64) {
        // if we had a logging system, we would log the error here
        let transactions =
            (self.transaction_repository.find_all(db_pool, filters).await).unwrap_or_default();
        let total = (self
            .transaction_repository
            .get_total(db_pool, filters)
            .await)
            .unwrap_or(0);

        (transactions, total)
    }

    pub async fn create(
        &self,
        db_pool: &PgPool,
        db_tx: &mut SqlxTransaction<'_, Postgres>,
        transaction: &TransactionCreate,
        current_user_id: &Uuid,
    ) -> anyhow::Result<Transaction> {
        self.account_repository
            .update_balance(
                db_pool,
                db_tx,
                transaction,
                transaction.amount,
                current_user_id,
            )
            .await?;

        let account = self
            .account_repository
            .find_by_id(db_pool, &transaction.to_account_id)
            .await?;

        let transaction = self
            .transaction_repository
            .create(db_pool, db_tx, &account, transaction)
            .await?;

        Ok(transaction)
    }
}
