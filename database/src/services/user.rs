use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    filters::user::Filter as UserFilter,
    models::user_dto::{User, UserCreate},
    repositories::users::UserRepository,
    traits::repository::Repository,
};

#[derive(Debug)]
pub struct Service {
    user_repository: UserRepository,
}

impl Service {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            user_repository: UserRepository::new(db_pool),
        }
    }

    pub async fn get_user_by_id<'a>(&self, db_pool: &'a PgPool, id: &Uuid) -> Option<User> {
        // if we had a logging system, we would log the error here
        match self.user_repository.find_by_id(db_pool, &id).await {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    pub async fn get_users<'a>(
        &self,
        db_pool: &'a PgPool,
        filters: &UserFilter,
    ) -> (Vec<User>, u64) {
        let users = match self.user_repository.find_all(filters).await {
            Ok(users) => users,
            Err(_) => Vec::<User>::new(),
        };
        let total = match self
            .user_repository
            .get_total(db_pool.clone(), filters)
            .await
        {
            Ok(total) => total,
            Err(_) => 0,
        };

        (users, total)
    }

    pub async fn get_user_by_email<'a>(&self, db_pool: &'a PgPool, email: &str) -> Option<User> {
        let filter = UserFilter {
            email: Some(email.to_string()),
            ..Default::default()
        };
        // if we had a logging system, we would log the error here
        match self
            .user_repository
            .find_one_by_filter(db_pool, &filter)
            .await
        {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    pub async fn create_user<'a>(
        &self,
        db_pool: &'a PgPool,
        user: UserCreate,
    ) -> anyhow::Result<User> {
        match self.user_repository.create(db_pool, &user).await {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }

    pub async fn update_user<'a>(
        &self,
        db_pool: &'a PgPool,
        id: &Uuid,
        user: &UserCreate,
    ) -> anyhow::Result<User> {
        match self.user_repository.update(db_pool, id, user).await {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }

    pub async fn delete_user<'a, E>(&self, db_pool: E, id: &Uuid) -> bool
    where
        E: Executor<'a, Database = Postgres> + Send + Sync + Clone,
    {
        self.user_repository.delete(db_pool, id).await
    }
}
