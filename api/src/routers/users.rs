use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use database::{
    filters::user::Filter as UserFilter,
    models::user_dto::{User, UserCreate},
    repositories::users::UserRepository,
    traits::repository::Repository,
};
use uuid::Uuid;

use crate::{
    http::response::{HttpPaginatedResponse, HttpResponse, ReturnTypes},
    state::application::ApplicationState,
};

pub fn get_router() -> Router<Arc<ApplicationState>> {
    Router::new()
        .route("/users", get(get_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
}

pub async fn get_users(
    State(state): State<Arc<ApplicationState>>,
    Query(mut filters): Query<UserFilter>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let db_pool = &state.db_pool;
    let repository = UserRepository::new(db_pool.clone());

    filters.enforce_pagination();

    match repository.find_all(&filters).await {
        Ok(users) => {
            let total = repository.get_total(&UserFilter::default()).await.unwrap();
            match filters.offset {
                Some(offset) => {
                    let paginated = HttpPaginatedResponse::new(users, offset, filters.limit, total);
                    Ok(Json(ReturnTypes::Paginated(paginated)))
                }
                None => Ok(Json(ReturnTypes::Multiple(users))),
            }
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))),
    }
}

pub async fn get_user(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let db_pool = &state.db_pool;
    let repository = UserRepository::new(db_pool.clone());
    match repository.find_by_id(&id).await {
        Ok(user) => match user {
            Some(user) => Ok(Json(ReturnTypes::Single(user))),
            None => Err((StatusCode::NOT_FOUND, Json("User not found".to_string()))),
        },
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))),
    }
}

pub async fn create_user(
    State(state): State<Arc<ApplicationState>>,
    Json(user): Json<UserCreate>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let db_pool = &state.db_pool;
    let repository = UserRepository::new(db_pool.clone());
    match repository.create(&user).await {
        Ok(user) => Ok(Json(ReturnTypes::Single(user))),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))),
    }
}

pub async fn update_user(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
    Json(user): Json<UserCreate>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let db_pool = &state.db_pool;
    let repository = UserRepository::new(db_pool.clone());
    match repository.update(&id, &user).await {
        Ok(user) => Ok(Json(ReturnTypes::Single(user))),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string()))),
    }
}

pub async fn delete_user(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<HttpResponse>, (StatusCode, Json<String>)> {
    let db_pool = &state.db_pool;
    let repository = UserRepository::new(db_pool.clone());
    match repository.delete(&id).await {
        true => Ok(Json(HttpResponse::new(
            StatusCode::OK.as_u16(),
            "User deleted successfully".to_string(),
            None,
        ))),
        false => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Internal Server Error".to_string()),
        )),
    }
}
