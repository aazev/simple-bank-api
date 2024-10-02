use crate::{
    filters::account::Filter as AccountFilter,
    models::{
        account_dto::{Account, AccountCreate},
        transaction_dto::{TransactionCreate, TransactionOperation},
        user_dto::User,
    },
};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AccountRepository;

impl Default for AccountRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountRepository {
    pub fn new() -> Self {
        Self
    }

    pub async fn find_all(
        &self,
        db_pool: &PgPool,
        filters: &AccountFilter,
    ) -> anyhow::Result<Vec<Account>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM accounts "#.to_owned() + &filters.query();

        let accounts = sqlx::query_as_with::<_, Account, _>(&query, args)
            .fetch_all(db_pool)
            .await?;

        Ok(accounts)
    }

    pub async fn find_one_by_filter(
        &self,
        db_pool: &PgPool,
        filters: &AccountFilter,
    ) -> anyhow::Result<Account> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM accounts "#.to_owned() + &filters.query();

        let account = sqlx::query_as_with::<_, Account, _>(&query, args)
            .fetch_optional(db_pool)
            .await?;

        if account.is_none() {
            return Err(anyhow::anyhow!("Account not found"));
        }

        Ok(account.unwrap())
    }

    pub async fn find_by_id(&self, db_pool: &PgPool, id: &Uuid) -> anyhow::Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            SELECT * FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(db_pool)
        .await?;

        Ok(account)
    }

    pub async fn create(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        user: &User,
        account: &AccountCreate,
    ) -> anyhow::Result<Account> {
        let account = account.to_account(user)?;

        sqlx::query(
            r#"
            INSERT INTO accounts (
                id,
                user_id,
                bank_id,
                bank_account_number,
                bank_account_digit,
                bank_agency_number,
                bank_agency_digit,
                bank_account_type,
                balance
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(account.id)
        .bind(account.user_id)
        .bind(account.bank_id)
        .bind(account.bank_account_number)
        .bind(account.bank_account_digit)
        .bind(account.bank_agency_number)
        .bind(account.bank_agency_digit)
        .bind(account.bank_account_type)
        .bind(&account.balance)
        .execute(&mut **executor)
        .await?;

        Ok(account)
    }

    pub async fn update(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &Uuid,
        account: &AccountCreate,
    ) -> anyhow::Result<Account> {
        let account = sqlx::query_as::<_, Account>(
            r#"
            UPDATE accounts
            SET
                user_id = $2,
                bank_id = $3,
                bank_account_number = $4,
                bank_account_digit = $5,
                bank_agency_number = $6,
                bank_agency_digit = $7,
                bank_account_type = $8,
                balance = $9
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(account.user_id)
        .bind(account.bank_id)
        .bind(account.bank_account_number)
        .bind(account.bank_account_digit)
        .bind(account.bank_agency_number)
        .bind(account.bank_agency_digit)
        .bind(account.bank_account_type)
        .bind(account.balance)
        .fetch_one(&mut **executor)
        .await?;

        Ok(account)
    }

    pub async fn update_balance(
        &self,
        db_pool: &PgPool,
        executor: &mut Transaction<'_, Postgres>,
        transaction: &TransactionCreate,
        amount: f64,
        acting_user_id: &Uuid,
    ) -> anyhow::Result<Account> {
        match &transaction.operation {
            TransactionOperation::Transfer => {
                let mut from_account = match transaction.from_account_id {
                    Some(from_account_id) => self.find_by_id(db_pool, &from_account_id).await?,
                    None => {
                        return Err(anyhow::anyhow!(
                            "Transfer operations must have a source account."
                        ));
                    }
                };

                if &from_account.user_id != acting_user_id {
                    return Err(anyhow::anyhow!(
                        "You are not allowed to perform this operation"
                    ));
                }

                let mut to_account = self.find_by_id(db_pool, &transaction.to_account_id).await?;

                let from_user = sqlx::query_as::<_, User>(r#"SELECT * FROM users WHERE id = $1"#)
                    .bind(from_account.user_id)
                    .fetch_one(&mut **executor)
                    .await?;
                let to_user = sqlx::query_as::<_, User>(r#"SELECT * FROM users WHERE id = $1"#)
                    .bind(to_account.user_id)
                    .fetch_one(&mut **executor)
                    .await?;

                let from_balance = from_account.get_balance(&from_user)?;
                let to_balance = to_account.get_balance(&to_user)?;

                let new_from_balance = from_balance - amount;
                let new_to_balance = to_balance + amount;

                from_account.update_balance(&from_user, new_from_balance)?;
                to_account.update_balance(&to_user, new_to_balance)?;

                sqlx::query(r#"UPDATE accounts SET balance = $2 WHERE id = $1"#)
                    .bind(from_account.id)
                    .bind(from_account.balance)
                    .execute(&mut **executor)
                    .await?;

                let to_account = sqlx::query_as::<_, Account>(
                    r#"UPDATE accounts SET balance = $2 WHERE id = $1 RETURNING *"#,
                )
                .bind(to_account.id)
                .bind(to_account.balance)
                .fetch_one(&mut **executor)
                .await?;

                Ok(to_account)
            }
            _ => {
                let mut to_account = self.find_by_id(db_pool, &transaction.to_account_id).await?;
                let user = sqlx::query_as::<_, User>(r#"SELECT * FROM users WHERE id = $1"#)
                    .bind(to_account.user_id)
                    .fetch_one(&mut **executor)
                    .await?;

                let balance = to_account.get_balance(&user)?;
                let new_balance = match &transaction.operation {
                    TransactionOperation::Deposit => balance + amount,
                    TransactionOperation::Fee => balance - amount,
                    TransactionOperation::Interest => balance + amount,
                    TransactionOperation::Payment => balance - amount,
                    TransactionOperation::Withdrawal => balance - amount,
                    _ => return Err(anyhow::anyhow!("Invalid operation")),
                };

                to_account.update_balance(&user, new_balance)?;

                let account = sqlx::query_as::<_, Account>(
                    r#"UPDATE accounts SET balance = $2 WHERE id = $1 RETURNING *"#,
                )
                .bind(to_account.id)
                .bind(to_account.balance)
                .fetch_one(&mut **executor)
                .await?;

                Ok(account)
            }
        }
    }

    pub async fn delete(&self, executor: &mut Transaction<'_, Postgres>, id: &Uuid) -> bool {
        sqlx::query!(
            r#"
            DELETE FROM accounts
            WHERE id = $1
            "#,
            id
        )
        .execute(&mut **executor)
        .await
        .is_ok()
    }

    pub async fn get_total(
        &self,
        db_pool: &PgPool,
        filters: &AccountFilter,
    ) -> anyhow::Result<u64> {
        let args = filters.get_arguments();
        let query = r#"SELECT COUNT(*) as total FROM accounts "#.to_owned() + &filters.total();
        let result = sqlx::query_with(&query, args).fetch_one(db_pool).await?;

        Ok(result.get::<i64, &str>("total") as u64)
    }
}
