use crate::{
    filters::account::Filter as AccountFilter,
    models::account_dto::{Account, AccountCreate},
};
use async_trait::async_trait;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::traits::repository::Repository;

use super::users::UserRepository;

#[derive(Debug, Clone)]
pub struct AccountRepository;
impl AccountRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Repository<Uuid, Account, AccountCreate, AccountFilter> for AccountRepository {
    async fn find_all(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &AccountFilter,
    ) -> anyhow::Result<Vec<Account>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM accounts "#.to_owned() + &filters.query();

        let accounts = sqlx::query_as_with::<_, Account, _>(&query, args)
            .fetch_all(*executor)
            .await?;

        Ok(accounts)
    }

    async fn find_one_by_filter(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &AccountFilter,
    ) -> anyhow::Result<Account> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM accounts "#.to_owned() + &filters.query();

        let account = sqlx::query_as_with::<_, Account, _>(&query, args)
            .fetch_optional(*executor)
            .await?;

        if account.is_none() {
            return Err(anyhow::anyhow!("Account not found"));
        }

        Ok(account.unwrap())
    }

    async fn find_by_id(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &Uuid,
    ) -> anyhow::Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT * FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(*executor)
        .await?;

        Ok(account)
    }

    async fn create(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        account: &AccountCreate,
    ) -> anyhow::Result<Account> {
        let user_repository = UserRepository::new(self.db_pool.clone());
        let mut tx = self.db_pool.begin().await?;
        let user = user_repository.find_by_id(&account.user_id).await?;

        let account = account.to_account(&user)?;

        sqlx::query(
            r#"
            INSERT INTO accounts (id, user_id, balance)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(&account.id)
        .bind(&account.user_id)
        .bind(&account.balance)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(account)
    }

    async fn update(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &Uuid,
        account: &AccountCreate,
    ) -> anyhow::Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            UPDATE accounts
            SET user_id = $2, balance = $3
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&account.user_id)
        .bind(&account.balance)
        .fetch_one(*executor)
        .await?;

        Ok(account)
    }

    async fn delete(&self, executor: &mut Transaction<'_, Postgres>, id: &Uuid) -> bool {
        sqlx::query!(
            r#"
            DELETE FROM accounts
            WHERE id = $1
            "#,
            id
        )
        .execute(*executor)
        .await
        .is_ok()
    }

    async fn get_total(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &AccountFilter,
    ) -> anyhow::Result<u64> {
        let args = filters.get_arguments();
        let query = r#"SELECT COUNT(*) as total FROM accounts "#.to_owned() + &filters.total();
        let result = sqlx::query_with(&query, args).fetch_one(*executor).await?;

        Ok(result.get::<i64, &str>("total") as u64)
    }
}
