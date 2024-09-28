use sqlx::{Postgres, Transaction as SqlxTransaction};
use uuid::Uuid;

use crate::{
    filters::{account::Filter as AccountFilter, user::Filter as UserFilter},
    models::{
        account_dto::{Account, AccountCreate},
        transaction_dto::{TransactionCreate, TransactionOperation},
    },
    repositories::{
        accounts::AccountRepository, transactions::TransactionRepository, users::UserRepository,
    },
};

#[derive(Debug)]
pub struct Service {
    transaction_repository: TransactionRepository,
    account_repository: AccountRepository,
    user_repository: UserRepository,
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
            user_repository: UserRepository::new(),
        }
    }

    pub async fn get_one_by_id(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        id: &Uuid,
    ) -> Option<Account> {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_by_id(executor, id).await {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn get_all(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        filters: &AccountFilter,
    ) -> (Vec<Account>, u64) {
        // if we had a logging system, we would log the error here
        let accounts = (self.account_repository.find_all(tx, filters).await).unwrap_or_default();
        let total = (self.account_repository.get_total(tx, filters).await).unwrap_or(0);

        (accounts, total)
    }

    pub async fn get_one_by_user_id(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        user_id: &Uuid,
    ) -> Option<Account> {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_by_id(tx, user_id).await {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn get_one_by_user_email(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        email: &str,
    ) -> Option<Account> {
        let filter = UserFilter {
            email: Some(email.to_string()),
            ..Default::default()
        };
        let user = match self.user_repository.find_one_by_filter(tx, &filter).await {
            Ok(user) => match user {
                Some(user) => user,
                None => return None,
            },
            Err(_) => return None,
        };

        let filter = AccountFilter {
            user_id: Some(user.id),
            ..Default::default()
        };
        // if we had a logging system, we would log the error here
        match self
            .account_repository
            .find_one_by_filter(tx, &filter)
            .await
        {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn create(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        user_id: &Uuid,
        account: AccountCreate,
    ) -> anyhow::Result<Account> {
        if account.balance < 0.0 {
            return Err(anyhow::anyhow!("Balance cannot be negative"));
        }

        let initial_balance = account.balance;

        let user = self.user_repository.find_by_id(tx, user_id).await?;

        let account = self.account_repository.create(tx, &user, &account).await?;

        let transaction = TransactionCreate {
            from_account_id: None,
            to_account_id: account.id,
            amount: initial_balance,
            operation: TransactionOperation::Deposit,
        };

        self.transaction_repository
            .create(tx, &account, &transaction)
            .await?;

        Ok(account)
    }

    pub async fn delete(&self, tx: &mut SqlxTransaction<'_, Postgres>, id: &Uuid) -> bool {
        match self
            .transaction_repository
            .delete_by_account_id(tx, id)
            .await
        {
            Ok(_) => {}
            Err(_) => return false,
        }

        self.account_repository.delete(tx, id).await
    }
}
