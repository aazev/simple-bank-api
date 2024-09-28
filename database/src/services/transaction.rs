use sqlx::{Postgres, Transaction as SqlxTransaction};
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

    pub async fn get_one_by_id(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        id: &Uuid,
    ) -> Option<Transaction> {
        // if we had a logging system, we would log the error here
        match self.transaction_repository.find_by_id(executor, id).await {
            Ok(transaction) => Some(transaction),
            Err(_) => None,
        }
    }

    pub async fn get_all(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &TransactionFilter,
    ) -> (Vec<Transaction>, u64) {
        // if we had a logging system, we would log the error here
        let transactions = (self
            .transaction_repository
            .find_all(executor, filters)
            .await)
            .unwrap_or_default();
        let total = (self
            .transaction_repository
            .get_total(executor, filters)
            .await)
            .unwrap_or(0);

        (transactions, total)
    }

    pub async fn create(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        transaction: &TransactionCreate,
    ) -> anyhow::Result<Transaction> {
        self.account_repository
            .update_balance(executor, transaction, transaction.amount)
            .await?;

        let account = self
            .account_repository
            .find_by_id(executor, &transaction.to_account_id)
            .await?;

        let transaction = self
            .transaction_repository
            .create(executor, &account, transaction)
            .await?;

        Ok(transaction)
    }
}
