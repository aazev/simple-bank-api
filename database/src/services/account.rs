use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::{
    filters::{account::Filter as AccountFilter, user::Filter as UserFilter},
    models::account_dto::{Account, AccountCreate},
    repositories::{accounts::AccountRepository, users::UserRepository},
};

#[derive(Debug)]
pub struct Service {
    account_repository: AccountRepository,
    user_repository: UserRepository,
}

impl Service {
    pub fn new() -> Self {
        Self {
            account_repository: AccountRepository::new(),
            user_repository: UserRepository::new(),
        }
    }

    pub async fn get_account_by_id<'a, E>(&self, executor: E, id: Uuid) -> Option<Account>
    where
        E: Executor<'a, Database = Postgres> + Send + Sync + Clone,
    {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_by_id(executor, &id).await {
            Ok(account) => Some(account),
            Err(_) => None,
        }
    }

    pub async fn get_accounts<'a, E>(&self, executor: E, filters: &AccountFilter) -> Vec<Account>
    where
        E: Executor<'a, Database = Postgres> + Send + Sync + Clone,
    {
        // if we had a logging system, we would log the error here
        match self.account_repository.find_all(executor, filters).await {
            Ok(accounts) => accounts,
            Err(_) => Vec::<Account>::new(),
        }
    }

    pub async fn get_account_by_user_id<'a, E>(&self, executor: E, user_id: Uuid) -> Option<Account>
    where
        E: Executor<'a, Database = Postgres> + Send + Sync + Clone,
    {
        let filter = AccountFilter {
            user_id: Some(user_id),
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

    pub async fn get_account_by_user_email<'a, E>(
        &self,
        executor: E,
        email: &str,
    ) -> Option<Account>
    where
        E: Executor<'a, Database = Postgres> + Send + Sync + Clone,
    {
        let filter = UserFilter {
            email: Some(email.to_string()),
            ..Default::default()
        };
        let user = match self
            .user_repository
            .find_one_by_filter(executor.clone(), &filter)
            .await
        {
            Ok(user) => user,
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

    pub async fn create_account<'a, E>(
        &self,
        executor: E,
        user: &Uuid,
        account: AccountCreate,
    ) -> anyhow::Result<Account>
    where
        E: Executor<'a, Database = Postgres> + Send + Sync + Clone,
    {
        if account.balance < 0.0 {
            return Err(anyhow::anyhow!("Balance cannot be negative"));
        }
        let initial_balance = account.balance;

        let user = match self
            .user_repository
            .find_by_id(executor.clone(), user)
            .await
        {
            Ok(user) => user,
            Err(e) => return Err(e),
        };

        let account = account.to_account(&user)?;

        match self.account_repository.create(executor, &account).await {
            Ok(account) => Ok(account),
            Err(e) => Err(e),
        }
    }
}
