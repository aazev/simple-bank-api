use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use database::{
    filters::transaction::Filter as TransactionFilter,
    models::{
        transaction_dto::{TransactionCreate, TransactionModel},
        user_dto::User,
    },
    repositories::{
        accounts::AccountRepository, transactions::TransactionRepository, users::UserRepository,
    },
    traits::repository::Repository,
};
use futures::{stream, StreamExt};
use uuid::Uuid;

use crate::{
    http::response::{HttpPaginatedResponse, HttpResponse, ReturnTypes},
    state::application::ApplicationState,
};

pub fn get_router() -> Router<Arc<ApplicationState>> {
    Router::new().route(
        "/accounts/:id/transactions",
        get(get_account_transactions).post(create_account_transaction),
    )
    // .route(
    //     "/accounts/:id/transactions/:transaction_id",
    //     get(get_account_transaction).delete(delete_account_transaction),
    // )
}

pub async fn get_account_transactions(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(account_id): Path<Uuid>,
    Query(mut filters): Query<TransactionFilter>,
) -> Result<Json<ReturnTypes<TransactionModel>>, (StatusCode, Json<HttpResponse>)> {
    let transaction_repository = TransactionRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());
    let account_repository = AccountRepository::new(state.db_pool.clone());

    let account = match account_repository.find_by_id(&account_id).await {
        Ok(account) => account,
        Err(err) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(HttpResponse::new(
                    StatusCode::NOT_FOUND.as_u16(),
                    err.to_string(),
                    None,
                )),
            ))
        }
    };

    if !scopes.contains(&"admin".to_string()) && account.user_id != current_user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(HttpResponse::new(
                StatusCode::FORBIDDEN.as_u16(),
                "Forbidden".to_string(),
                None,
            )),
        ));
    }
    filters.to_account_id = Some(account.id);
    filters.enforce_pagination();

    match transaction_repository.find_all(&filters).await {
        Ok(transactions) => {
            let total = transaction_repository
                .get_total(&TransactionFilter::default())
                .await
                .unwrap();
            let transaction_models = stream::iter(transactions)
                .enumerate()
                .map(|(_index, transaction)| {
                    let account_repository = account_repository.clone();
                    let user_repository = user_repository.clone();
                    async move {
                        let account = account_repository
                            .find_by_id(&transaction.to_account_id)
                            .await
                            .unwrap();
                        let user = user_repository.find_by_id(&account.user_id).await.unwrap();
                        TransactionModel::from_dto(&transaction, &user.encryption_key).unwrap()
                    }
                })
                .buffered(10)
                .collect::<Vec<TransactionModel>>()
                .await;
            match filters.offset {
                Some(offset) => {
                    let paginated = HttpPaginatedResponse::new(
                        transaction_models,
                        offset,
                        filters.limit,
                        total,
                    );
                    Ok(Json(ReturnTypes::Paginated(paginated)))
                }
                None => Ok(Json(ReturnTypes::Multiple(transaction_models))),
            }
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HttpResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                err.to_string(),
                None,
            )),
        )),
    }
}

pub async fn create_account_transaction(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(account_id): Path<Uuid>,
    Json(transaction): Json<TransactionCreate>,
) -> Result<Json<ReturnTypes<TransactionModel>>, (StatusCode, Json<HttpResponse>)> {
    let transaction_repository = TransactionRepository::new(state.db_pool.clone());
    let user_repository = UserRepository::new(state.db_pool.clone());
    let account_repository = AccountRepository::new(state.db_pool.clone());

    let account = match account_repository.find_by_id(&account_id).await {
        Ok(account) => account,
        Err(err) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(HttpResponse::new(
                    StatusCode::NOT_FOUND.as_u16(),
                    err.to_string(),
                    None,
                )),
            ))
        }
    };

    if !scopes.contains(&"admin".to_string()) && account.user_id != current_user.id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(HttpResponse::new(
                StatusCode::FORBIDDEN.as_u16(),
                "Forbidden".to_string(),
                None,
            )),
        ));
    }

    let user = user_repository.find_by_id(&account.user_id).await.unwrap();

    match transaction_repository.create(&transaction).await {
        Ok(transaction) => {
            let transaction_model =
                TransactionModel::from_dto(&transaction, &user.encryption_key).unwrap();
            Ok(Json(ReturnTypes::Single(transaction_model)))
        }
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(HttpResponse::new(
                StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                err.to_string(),
                None,
            )),
        )),
    }
}
