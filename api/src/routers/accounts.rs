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
    repositories::{accounts::AccountRepository, users::UserRepository},
    traits::repository::Repository,
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
        .route(
            "/accounts/:id",
            get(get_account).put(update_account).delete(delete_account),
        )
}

pub async fn get_accounts(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Query(mut filters): Query<AccountFilter>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let account_repository = AccountRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());

    if !scopes.contains(&"admin".to_string()) {
        let user = match user_repository.find_by_id(&current_user.id).await {
            Ok(user) => user,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HttpResponse::new(
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        e.to_string(),
                        None,
                    )),
                ))
            }
        };
        filters.user_id = Some(user.id);
    }
    filters.enforce_pagination();

    match account_repository.find_all(&filters).await {
        Ok(accounts) => {
            let total = account_repository.get_total(&filters).await.unwrap();
            let account_models: Vec<AccountModel> = stream::iter(accounts)
                .enumerate()
                .map(|(_index, account)| {
                    let user_repository = user_repository.clone();
                    async move {
                        let user = user_repository.find_by_id(&account.user_id).await.unwrap();
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
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HttpResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                e.to_string(),
                None,
            )),
        )),
    }
}

pub async fn create_account(
    State(state): State<Arc<ApplicationState>>,
    Json(account): Json<AccountCreate>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let account_repository = AccountRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());

    let user = match user_repository.find_by_id(&account.user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(HttpResponse::new(
                    StatusCode::NOT_FOUND.as_u16(),
                    "User not found".to_string(),
                    None,
                )),
            ))
        }
    };

    match account_repository.create(&account).await {
        Ok(account) => {
            let account_model = AccountModel::from_dto(&account, &user).unwrap();
            Ok(Json(ReturnTypes::Single(account_model)))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HttpResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    e.to_string(),
                    None,
                )),
            ));
        }
    }
}

pub async fn get_account(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let account_repository = AccountRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());

    match account_repository.find_by_id(&id).await {
        Ok(account) => {
            if !scopes.contains(&"admin".to_string()) {
                let user = match user_repository.find_by_id(&current_user.id).await {
                    Ok(user) => user,
                    Err(e) => {
                        return Err((
                            StatusCode::NOT_FOUND,
                            Json(HttpResponse::new(
                                StatusCode::NOT_FOUND.as_u16(),
                                format!("User not found: {}", e),
                                None,
                            )),
                        ))
                    }
                };

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

            let user = user_repository.find_by_id(&account.user_id).await.unwrap();
            let account_model = AccountModel::from_dto(&account, &user).unwrap();
            Ok(Json(ReturnTypes::Single(account_model)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HttpResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                e.to_string(),
                None,
            )),
        )),
    }
}

pub async fn update_account(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(id): Path<Uuid>,
    Json(account_create): Json<AccountCreate>,
) -> Result<Json<ReturnTypes<AccountModel>>, (StatusCode, Json<HttpResponse>)> {
    let account_repository = AccountRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());

    match account_repository.find_by_id(&id).await {
        Ok(account) => {
            if !scopes.contains(&"admin".to_string()) {
                let user = match user_repository.find_by_id(&current_user.id).await {
                    Ok(user) => user,
                    Err(e) => {
                        return Err((
                            StatusCode::NOT_FOUND,
                            Json(HttpResponse::new(
                                StatusCode::NOT_FOUND.as_u16(),
                                format!("User not found: {}", e),
                                None,
                            )),
                        ))
                    }
                };

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

            match account_repository.update(&id, &account_create).await {
                Ok(account) => {
                    let user = user_repository.find_by_id(&account.user_id).await.unwrap();
                    let account_model = AccountModel::from_dto(&account, &user).unwrap();
                    Ok(Json(ReturnTypes::Single(account_model)))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HttpResponse::new(
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        e.to_string(),
                        None,
                    )),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(HttpResponse::new(
                StatusCode::NOT_FOUND.as_u16(),
                format!("Account not found: {}", e),
                None,
            )),
        )),
    }
}

pub async fn delete_account(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(id): Path<Uuid>,
) -> Result<Json<HttpResponse>, (StatusCode, Json<HttpResponse>)> {
    let account_repository = AccountRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());

    match account_repository.find_by_id(&id).await {
        Ok(account) => {
            if !scopes.contains(&"admin".to_string()) {
                let user = match user_repository.find_by_id(&current_user.id).await {
                    Ok(user) => user,
                    Err(e) => {
                        return Err((
                            StatusCode::NOT_FOUND,
                            Json(HttpResponse::new(
                                StatusCode::NOT_FOUND.as_u16(),
                                format!("User not found: {}", e),
                                None,
                            )),
                        ))
                    }
                };

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

            match account_repository.delete(&id).await {
                true => Ok(Json(HttpResponse::new(
                    StatusCode::NO_CONTENT.as_u16(),
                    "Account deleted".to_string(),
                    None,
                ))),
                false => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HttpResponse::new(
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        "Account not deleted".to_string(),
                        None,
                    )),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(HttpResponse::new(
                StatusCode::NOT_FOUND.as_u16(),
                format!("Account not found: {}", e),
                None,
            )),
        )),
    }
}
