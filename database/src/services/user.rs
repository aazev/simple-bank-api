use sqlx::{Postgres, Transaction as SqlxTransaction};
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

    pub async fn get_all(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        filters: &UserFilter,
    ) -> (Vec<User>, u64) {
        let users = (self.user_repository.find_all(tx, filters).await).unwrap_or_default();
        let total = (self.user_repository.get_total(tx, filters).await).unwrap_or(0);

        (users, total)
    }

    pub async fn get_one_by_id(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        id: &Uuid,
    ) -> Option<User> {
        // if we had a logging system, we would log the error here
        match self.user_repository.find_by_id(tx, id).await {
            Ok(user) => Some(user),
            Err(_) => None,
        }
    }

    pub async fn get_one_by_email(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        email: &str,
    ) -> Option<User> {
        let filter = UserFilter {
            email: Some(email.to_string()),
            ..Default::default()
        };
        // if we had a logging system, we would log the error here
        (self.user_repository.find_one_by_filter(tx, &filter).await).unwrap_or_default()
    }

    pub async fn get_one_by_filter(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        filter: &UserFilter,
    ) -> Option<User> {
        // if we had a logging system, we would log the error here
        (self.user_repository.find_one_by_filter(tx, filter).await).unwrap_or_default()
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
