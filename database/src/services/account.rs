use sqlx::{Postgres, Transaction as SqlxTransaction};
use uuid::Uuid;

use crate::{
    filters::{account::Filter as AccountFilter, user::Filter as UserFilter},
    models::{
        account_dto::{Account, AccountCreate},
        transaction_dto::{TransactionCreate, TransactionType},
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
        id: Uuid,
    ) -> Option<Account> {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_by_id(executor, &id).await {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn get_accounts(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        filters: &AccountFilter,
    ) -> Vec<Account> {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_all(executor, filters).await {
            Ok(accounts) => accounts,
            Err(_) => Vec::<Account>::new(),
        }
    }

    pub async fn get_one_by_user_id(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        user_id: &Uuid,
    ) -> Option<Account> {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_by_id(executor, user_id).await {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn get_one_by_user_email(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        email: &str,
    ) -> Option<Account> {
        let filter = UserFilter {
            email: Some(email.to_string()),
            ..Default::default()
        };
        let user = match self
            .user_repository
            .find_one_by_filter(executor, &filter)
            .await
        {
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
            .find_one_by_filter(executor, &filter)
            .await
        {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn create(
        &self,
        executor: &mut SqlxTransaction<'_, Postgres>,
        user_id: &Uuid,
        account: AccountCreate,
    ) -> anyhow::Result<Account> {
        if account.balance < 0.0 {
            return Err(anyhow::anyhow!("Balance cannot be negative"));
        }

        let initial_balance = account.balance;

        let user = self.user_repository.find_by_id(executor, user_id).await?;

        let account = self
            .account_repository
            .create(executor, &user, &account)
            .await?;

        let transaction = TransactionCreate {
            from_account_id: None,
            to_account_id: account.id,
            amount: initial_balance,
            _type: TransactionType::Deposit,
        };

        self.transaction_repository
            .create(executor, &transaction)
            .await?;

        Ok(account)
    }
}
