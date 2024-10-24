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
    services::user::Service as UserService,
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

#[utoipa::path(
    get,
    path = "/users",
    context_path = "/api/v1",
    params(
        ("id" = Option<Uuid>, Query, description = "User ID"),
        ("name" = Option<String>, Query, description = "User name"),
        ("email" = Option<String>, Query, description = "User email"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit"),
    ),
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<User>),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn get_users(
    State(state): State<Arc<ApplicationState>>,
    Query(mut filters): Query<UserFilter>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<HttpResponse>)> {
    let user_service = UserService::new();

    filters.enforce_pagination();
    let (users, total) = user_service.get_all(&state.db_pool, &filters).await;

    match filters.offset {
        Some(offset) => Ok(Json(ReturnTypes::Paginated(HttpPaginatedResponse::new(
            users,
            offset,
            filters.limit,
            total,
        )))),
        None => Ok(Json(ReturnTypes::Multiple(users))),
    }
}

#[utoipa::path(
    get,
    path = "/users/:id",
    context_path = "/api/v1",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<User>),
        (status = 404, description = "User not found", body = HttpResponse, example = json!(r#"{"status": 404, "message": "User not found"}"#)),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn get_user(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let user_service = UserService::new();

    match user_service.get_one_by_id(&state.db_pool, &id).await {
        Some(user) => Ok(Json(ReturnTypes::Single(user))),
        None => Err((StatusCode::NOT_FOUND, Json("User not found".to_string()))),
    }
}

#[utoipa::path(
    post,
    path = "/users",
    context_path = "/api/v1",
    request_body = UserCreate,
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<User>),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn create_user(
    State(state): State<Arc<ApplicationState>>,
    Json(user): Json<UserCreate>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let user_service = UserService::new();
    let mut tx = state.db_pool.begin().await.unwrap();

    match user_service.create(&mut tx, &user).await {
        Ok(user) => {
            tx.commit().await.unwrap();
            Ok(Json(ReturnTypes::Single(user)))
        }
        Err(err) => {
            tx.rollback().await.unwrap();
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())))
        }
    }
}

#[utoipa::path(
    put,
    path = "/users/:id",
    context_path = "/api/v1",
    params(("id" = Uuid, Path, description = "User ID")),
    request_body = UserCreate,
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<User>),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn update_user(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
    Json(user): Json<UserCreate>,
) -> Result<Json<ReturnTypes<User>>, (StatusCode, Json<String>)> {
    let user_service = UserService::new();
    let mut tx = state.db_pool.begin().await.unwrap();

    match user_service.update(&mut tx, &id, &user).await {
        Ok(user) => {
            tx.commit().await.unwrap();
            Ok(Json(ReturnTypes::Single(user)))
        }
        Err(err) => {
            tx.rollback().await.unwrap();
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/users/:id",
    context_path = "/api/v1",
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "Successful response", body = HttpResponse, example = json!(r#"{"status": 200, "message": "User deleted successfully"}"#)),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn delete_user(
    State(state): State<Arc<ApplicationState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<HttpResponse>, (StatusCode, Json<String>)> {
    let user_service = UserService::new();
    let mut tx = state.db_pool.begin().await.unwrap();

    match user_service.delete(&mut tx, &id).await {
        true => {
            tx.commit().await.unwrap();
            Ok(Json(HttpResponse::new(
                StatusCode::OK.as_u16(),
                "User deleted successfully".to_string(),
                None,
            )))
        }
        false => {
            tx.rollback().await.unwrap();
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal Server Error".to_string()),
            ))
        }
    }
}
