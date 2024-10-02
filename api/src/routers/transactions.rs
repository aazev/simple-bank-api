use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use database::{
    filters::transaction::Filter as TransactionFilter,
    models::{
        transaction_dto::{TransactionCreate, TransactionModel, TransactionOperation},
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
    Router::new()
        .route("/transactions", post(create_account_transaction))
        .route("/accounts/:id/transactions", get(get_account_transactions))
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
    let transaction_service = TransactionService::new();
    let account_service = AccountService::new();

    let account = match account_service
        .get_one_by_id(&state.db_pool, &account_id)
        .await
    {
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
    filters.account_id = Some(account.id);
    filters.enforce_pagination();

    let (transactions, total) = transaction_service.get_all(&state.db_pool, &filters).await;
    let transaction_models = stream::iter(transactions)
        .enumerate()
        .map(|(_index, transaction)| {
            let db_pool = state.db_pool.clone();
            let account_service = AccountService::new();
            let user_service = UserService::new();
            async move {
                let to_account = account_service
                    .get_one_by_id(&db_pool, &transaction.to_account_id)
                    .await
                    .unwrap();
                let user_to = user_service
                    .get_one_by_id(&db_pool, &to_account.user_id)
                    .await
                    .unwrap();
                let res = TransactionModel::from_dto(&transaction, &user_to.encryption_key);
                if res.is_err() && transaction.from_account_id.is_some() {
                    dbg!(format!("Error: {:?}", res));
                    let from_account = account_service
                        .get_one_by_id(&db_pool, &transaction.from_account_id.unwrap())
                        .await
                        .unwrap();
                    let user_from = user_service
                        .get_one_by_id(&db_pool, &from_account.user_id)
                        .await
                        .unwrap();

                    let res_2 = TransactionModel::from_dto(&transaction, &user_from.encryption_key);

                    dbg!(format!("Error2: {:?}", res_2));

                    res_2
                } else {
                    res
                }
            }
        })
        .buffered(10)
        .collect::<Vec<Result<TransactionModel, _>>>()
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .collect::<Vec<TransactionModel>>();

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
    Json(transaction): Json<TransactionCreate>,
) -> Result<Json<ReturnTypes<TransactionModel>>, (StatusCode, Json<HttpResponse>)> {
    let mut tx = state.db_pool.begin().await.unwrap();
    let transaction_service = TransactionService::new();
    let account_service = AccountService::new();
    let user_service = UserService::new();

    let to_account = match account_service
        .get_one_by_id(&state.db_pool, &transaction.to_account_id)
        .await
    {
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

    match transaction.operation {
        TransactionOperation::Withdrawal => {
            if to_account.user_id != current_user.id {
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
        TransactionOperation::Transfer => {
            let from_account_id = match &transaction.from_account_id {
                Some(id) => *id,
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(HttpResponse::new(
                            StatusCode::BAD_REQUEST.as_u16(),
                            "Transfer transactions need an origin account".to_string(),
                            None,
                        )),
                    ))
                }
            };
            let from_account = match account_service
                .get_one_by_id(&state.db_pool, &from_account_id)
                .await
            {
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

            if from_account.user_id != current_user.id {
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
        TransactionOperation::Interest | TransactionOperation::Fee => {
            if !scopes.contains(&"admin".to_string()) {
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
        _ => {}
    }

    let user = user_service
        .get_one_by_id(&state.db_pool, &to_account.user_id)
        .await
        .unwrap();
    match transaction_service
        .create(&state.db_pool, &mut tx, &transaction, &current_user.id)
        .await
    {
        Ok(transaction) => match TransactionModel::from_dto(&transaction, &user.encryption_key) {
            Ok(transaction_model) => {
                tx.commit().await.unwrap();
                Ok(Json(ReturnTypes::Single(transaction_model)))
            }
            Err(e) => {
                tx.rollback().await.unwrap();
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(HttpResponse::new(
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        format!("Error creating transaction: {}", e),
                        None,
                    )),
                ))
            }
        },
        Err(e) => {
            tx.rollback().await.unwrap();
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(HttpResponse::new(
                    StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    format!("Error creating transaction: {}", e),
                    None,
                )),
            ))
        }
    }
}
