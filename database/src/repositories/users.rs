use crate::{filters::user::Filter as UserFilter, models::user_dto::UserCreate};
use async_trait::async_trait;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{models::user_dto::User, traits::repository::Repository};

#[derive(Debug, Clone)]
pub struct UserRepository;
impl UserRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Repository<Uuid, User, UserCreate, UserFilter> for UserRepository {
    async fn find_all(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &UserFilter,
    ) -> anyhow::Result<Vec<User>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM users "#.to_owned() + &filters.query();

        let users = sqlx::query_as_with::<_, User, _>(&query, args)
            .fetch_all(&mut **executor)
            .await?;

        Ok(users)
    }

    async fn find_by_id(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &Uuid,
    ) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(&mut **executor)
        .await?;

        Ok(user)
    }

    async fn find_one_by_filter(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &UserFilter,
    ) -> anyhow::Result<Option<User>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM users "#.to_owned() + &filters.query();

        let user = sqlx::query_as_with::<_, User, _>(&query, args)
            .fetch_optional(&mut **executor)
            .await?;

        Ok(user)
    }

    async fn create(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        entity: &UserCreate,
    ) -> anyhow::Result<User> {
        let new_user = User::try_from(entity)?;
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, name, email, password, encryption_key)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            new_user.id,
            new_user.name,
            new_user.email,
            new_user.password,
            new_user.encryption_key
        )
        .fetch_one(&mut **executor)
        .await?;

        Ok(user)
    }

    async fn update(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        id: &Uuid,
        entity: &UserCreate,
    ) -> anyhow::Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET name = $1, email = $2
            WHERE id = $3
            RETURNING *
            "#,
            entity.name,
            entity.email,
            id
        )
        .fetch_one(&mut **executor)
        .await?;

        Ok(user)
    }

    async fn delete(&self, executor: &mut Transaction<'_, Postgres>, id: &Uuid) -> bool {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(&mut **executor)
        .await
        .is_ok()
    }

    async fn get_total(
        &self,
        executor: &mut Transaction<'_, Postgres>,
        filters: &UserFilter,
    ) -> anyhow::Result<u64> {
        let args = filters.get_arguments();
        let query = r#"SELECT COUNT(*) as total FROM users "#.to_owned() + &filters.total();
        let result = sqlx::query_with(&query, args)
            .fetch_one(&mut **executor)
            .await?;

        Ok(result.get::<i64, &str>("total") as u64)
    }
}
