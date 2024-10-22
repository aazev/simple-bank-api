use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use database::{
    filters::account::Filter as AccountFilter,
    models::{
        account_dto::{AccountCreate, AccountModel},
        user_dto::User,
    },
    services::{account::Service as AccountService, user::Service as UserService},
};
use futures::{stream, StreamExt};
use uuid::Uuid;

use crate::{
    http::response::{HttpPaginatedResponse, HttpResponse, ReturnTypes},
    state::application::ApplicationState,
};

pub fn get_router() -> Router<Arc<ApplicationState>> {
    Router::new()
        .route("/accounts", get(get_accounts).post(create_account))
        .route("/accounts/:id", get(get_account).delete(delete_account))
}

#[utoipa::path(
    get,
    path = "/accounts",
    context_path = "/api/v1",
    params(
        ("id" = Option<Uuid>, Query, description = "Account ID"),
        ("user_id" = Option<Uuid>, Query, description = "User ID"),
        ("offset" = Option<usize>, Query, description = "Pagination offset"),
        ("limit" = Option<usize>, Query, description = "Pagination limit"),
    ),
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<AccountModel>),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn get_accounts(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Query(mut filters): Query<AccountFilter>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let account_service = AccountService::new();
    let user_service = UserService::new();

    if !scopes.contains(&"admin".to_string()) {
        let user = user_service
            .get_one_by_id(&state.db_pool, &current_user.id)
            .await
            .unwrap();
        filters.user_id = Some(user.id);
    }
    filters.enforce_pagination();

    let (accounts, total) = account_service.get_all(&state.db_pool, &filters).await;

    let account_models: Vec<AccountModel> = stream::iter(accounts)
        .enumerate()
        .map(|(_index, account)| {
            let user_service = UserService::new();
            let db_pool = state.db_pool.clone();
            async move {
                let db_pool = db_pool.clone();
                let user = user_service
                    .get_one_by_id(&db_pool, &account.user_id)
                    .await
                    .unwrap();
                AccountModel::from_dto(&account, &user).unwrap()
            }
        })
        .buffered(10)
        .collect::<Vec<AccountModel>>()
        .await;

    match filters.offset {
        Some(offset) => {
            let paginated =
                HttpPaginatedResponse::new(account_models, offset, filters.limit, total);
            Ok(Json(ReturnTypes::Paginated(paginated)))
        }
        None => Ok(Json(ReturnTypes::Multiple(account_models))),
    }
}

#[utoipa::path(
    post,
    path = "/accounts",
    context_path = "/api/v1",
    request_body = AccountCreate,
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<AccountModel>),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn create_account(
    State(state): State<Arc<ApplicationState>>,
    Json(account): Json<AccountCreate>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let mut tx = state.db_pool.begin().await.unwrap();

    let account_service = AccountService::new();
    let user_service = UserService::new();

    let user = user_service
        .get_one_by_id(&state.db_pool, &account.user_id)
        .await
        .unwrap();

    match account_service
        .create(&state.db_pool, &mut tx, &account.user_id.clone(), account)
        .await
    {
        Ok(account) => {
            let account_model = AccountModel::from_dto(&account, &user).unwrap();
            tx.commit().await.unwrap();
            Ok(Json(ReturnTypes::Single(account_model)))
        }
        Err(e) => {
            tx.rollback().await.unwrap();
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HttpResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    e.to_string(),
                    None,
                )),
            ))
        }
    }
}

#[utoipa::path(
    get,
    path = "/accounts/:id",
    context_path = "/api/v1",
    params(("id" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 200, description = "Successful response", body = ReturnTypes<AccountModel>),
        (status = 404, description = "Account not found", body = HttpResponse, example = json!(r#"{"status": 404, "message": "Account not found"}"#)),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn get_account(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let account_service = AccountService::new();
    let user_service = UserService::new();

    match account_service.get_one_by_id(&state.db_pool, &id).await {
        Some(account) => {
            if !scopes.contains(&"admin".to_string()) {
                let user = user_service
                    .get_one_by_id(&state.db_pool, &current_user.id)
                    .await
                    .unwrap();

                if user.id != account.user_id {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(HttpResponse::new(
                            StatusCode::FORBIDDEN.as_u16(),
                            "Forbidden".to_string(),
                            None,
                        )),
                    ));
                }
            }

            let user = user_service
                .get_one_by_id(&state.db_pool, &account.user_id)
                .await
                .unwrap();
            let account_model = AccountModel::from_dto(&account, &user).unwrap();
            Ok(Json(ReturnTypes::Single(account_model)))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(HttpResponse::new(
                StatusCode::NOT_FOUND.as_u16(),
                "Account not found".to_string(),
                None,
            )),
        )),
    }
}

#[utoipa::path(
    delete,
    path = "/accounts/:id",
    context_path = "/api/v1",
    params(("id" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 204, description = "Account deleted", body = HttpResponse, example = json!(r#"{"status": 200, "message": "Account deleted"}"#)),
        (status = 403, description = "Forbidden", body = HttpResponse, example = json!(r#"{"status": 403, "message": "Forbidden"}"#)),
        (status = 404, description = "Account not found", body = HttpResponse, example = json!(r#"{"status": 404, "message": "Account not found"}"#)),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn delete_account(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(id): Path<Uuid>,
) -> Result<Json<HttpResponse>, (StatusCode, Json<HttpResponse>)> {
    let mut tx = state.db_pool.begin().await.unwrap();
    let user_service = UserService::new();
    let account_service = AccountService::new();

    match account_service.get_one_by_id(&state.db_pool, &id).await {
        Some(account) => {
            if !scopes.contains(&"admin".to_string()) {
                let user = user_service
                    .get_one_by_id(&state.db_pool, &current_user.id)
                    .await
                    .unwrap();

                if user.id != account.user_id {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(HttpResponse::new(
                            StatusCode::FORBIDDEN.as_u16(),
                            "Forbidden".to_string(),
                            None,
                        )),
                    ));
                }
            }

            match account_service.delete(&mut tx, &id).await {
                true => {
                    tx.commit().await.unwrap();
                    Ok(Json(HttpResponse::new(
                        StatusCode::OK.as_u16(),
                        "Account deleted".to_string(),
                        None,
                    )))
                }
                false => {
                    tx.rollback().await.unwrap();
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(HttpResponse::new(
                            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                            "Account not deleted".to_string(),
                            None,
                        )),
                    ))
                }
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(HttpResponse::new(
                StatusCode::NOT_FOUND.as_u16(),
                "Account not found".to_string(),
                None,
            )),
        )),
    }
}
