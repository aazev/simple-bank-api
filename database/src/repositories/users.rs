use crate::{filters::user::Filter as UserFilter, models::user_dto::UserCreate};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{models::user_dto::User, traits::repository::Repository};

#[derive(Debug)]
pub struct UserRepository {
    db_pool: PgPool,
}

impl UserRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}
#[async_trait]
impl Repository<Uuid, User, UserCreate, UserFilter> for UserRepository {
    async fn find_all(&self, filters: &UserFilter) -> anyhow::Result<Vec<User>> {
        let args = filters.get_arguments();
        let query = r#"SELECT * FROM users "#.to_owned() + &filters.query();

        dbg!(&query);

        let users = sqlx::query_as_with::<_, User, _>(&query, args)
            .fetch_all(&self.db_pool)
            .await?;

        Ok(users)
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(user)
    }

    async fn create(&self, entity: &UserCreate) -> anyhow::Result<User> {
        let new_user = User::try_from(entity)?;
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password, encryption_key)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            new_user.name,
            new_user.email,
            new_user.password,
            new_user.encryption_key
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(user)
    }

    async fn update(&self, id: &Uuid, entity: &UserCreate) -> anyhow::Result<User> {
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
        .fetch_one(&self.db_pool)
        .await?;

        Ok(user)
    }

    async fn delete(&self, id: &Uuid) -> bool {
        sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.db_pool)
        .await
        .is_ok()
    }

    async fn get_total(&self, filters: &UserFilter) -> anyhow::Result<u64> {
        let args = filters.get_arguments();
        let query = r#"SELECT COUNT(*) as total FROM users "#.to_owned() + &filters.total();
        let result = sqlx::query_with(&query, args)
            .fetch_one(&self.db_pool)
            .await?;

        Ok(result.get::<i64, &str>("total") as u64)
    }
}
