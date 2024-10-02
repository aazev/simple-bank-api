use sqlx::{PgPool, Postgres, Transaction as SqlxTransaction};
use uuid::Uuid;

use crate::{
    filters::user::Filter as UserFilter,
    models::user_dto::{User, UserCreate},
    repositories::users::UserRepository,
};

#[derive(Debug)]
pub struct Service {
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
            user_repository: UserRepository::new(),
        }
    }

    pub async fn get_all(&self, db_pool: &PgPool, filters: &UserFilter) -> (Vec<User>, u64) {
        let users = (self.user_repository.find_all(db_pool, filters).await).unwrap_or_default();
        let total = (self.user_repository.get_total(db_pool, filters).await).unwrap_or(0);

        (users, total)
    }

    pub async fn get_one_by_id(&self, db_pool: &PgPool, id: &Uuid) -> Option<User> {
        // if we had a logging system, we would log the error here
        match self.user_repository.find_by_id(db_pool, id).await {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    pub async fn get_one_by_email(&self, db_pool: &PgPool, email: &str) -> Option<User> {
        let filter = UserFilter {
            email: Some(email.to_string()),
            ..Default::default()
        };
        // if we had a logging system, we would log the error here
        (self
            .user_repository
            .find_one_by_filter(db_pool, &filter)
            .await)
            .unwrap_or_default()
    }

    pub async fn get_one_by_filter(&self, db_pool: &PgPool, filter: &UserFilter) -> Option<User> {
        // if we had a logging system, we would log the error here
        (self
            .user_repository
            .find_one_by_filter(db_pool, filter)
            .await)
            .unwrap_or_default()
    }

    pub async fn create(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        user: &UserCreate,
    ) -> anyhow::Result<User> {
        match self.user_repository.create(tx, user).await {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }

    pub async fn update(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        id: &Uuid,
        user: &UserCreate,
    ) -> anyhow::Result<User> {
        match self.user_repository.update(tx, id, user).await {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }

    pub async fn delete(&self, tx: &mut SqlxTransaction<'_, Postgres>, id: &Uuid) -> bool {
        self.user_repository.delete(tx, id).await
    }
}
