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
    services::{
        account::Service as AccountService, transaction::Service as TransactionService,
        user::Service as UserService,
    },
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
    let mut tx = state.db_pool.begin().await.unwrap();
    let transaction_service = TransactionService::new();
    let account_service = AccountService::new();

    let account = match account_service.get_one_by_id(&mut tx, &account_id).await {
        Some(account) => account,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(HttpResponse::new(
                    StatusCode::NOT_FOUND.as_u16(),
                    "Account not found".to_string(),
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

    let (transactions, total) = transaction_service.get_all(&mut tx, &filters).await;
    let transaction_models = stream::iter(transactions)
        .enumerate()
        .map(|(_index, transaction)| {
            let db_pool = state.db_pool.clone();
            let account_service = AccountService::new();
            let user_service = UserService::new();
            async move {
                let mut tx = db_pool.begin().await.unwrap();
                let account = account_service
                    .get_one_by_id(&mut tx, &account_id)
                    .await
                    .unwrap();
                let user = user_service
                    .get_one_by_id(&mut tx, &account.user_id)
                    .await
                    .unwrap();
                TransactionModel::from_dto(&transaction, &user.encryption_key).unwrap()
            }
        })
        .buffered(10)
        .collect::<Vec<TransactionModel>>()
        .await;

    match filters.offset {
        Some(offset) => {
            let paginated =
                HttpPaginatedResponse::new(transaction_models, offset, filters.limit, total);
            Ok(Json(ReturnTypes::Paginated(paginated)))
        }
        None => Ok(Json(ReturnTypes::Multiple(transaction_models))),
    }
}

pub async fn create_account_transaction(
    State(state): State<Arc<ApplicationState>>,
    Extension(current_user): Extension<User>,
    Extension(scopes): Extension<Vec<String>>,
    Path(account_id): Path<Uuid>,
    Json(transaction): Json<TransactionCreate>,
) -> Result<Json<ReturnTypes<TransactionModel>>, (StatusCode, Json<HttpResponse>)> {
    let mut tx = state.db_pool.begin().await.unwrap();
    let transaction_service = TransactionService::new();
    let account_service = AccountService::new();
    let user_service = UserService::new();

    let account = match account_service.get_one_by_id(&mut tx, &account_id).await {
        Some(account) => account,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(HttpResponse::new(
                    StatusCode::NOT_FOUND.as_u16(),
                    "Account not found".to_string(),
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

    let user = user_service
        .get_one_by_id(&mut tx, &account.user_id)
        .await
        .unwrap();
    match transaction_service.create(&mut tx, &transaction).await {
        Ok(transaction) => {
            let transaction_model =
                TransactionModel::from_dto(&transaction, &user.encryption_key).unwrap();

            tx.commit().await.unwrap();
            Ok(Json(ReturnTypes::Single(transaction_model)))
        }
        Err(_) => {
            tx.rollback().await.unwrap();
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HttpResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "Error creating transaction".to_string(),
                    None,
                )),
            ))
        }
    }
}
