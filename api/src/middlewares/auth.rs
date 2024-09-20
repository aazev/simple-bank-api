use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use chrono::NaiveDateTime;
use database::{repositories::users::UserRepository, traits::repository::Repository};
use hmac::{digest::KeyInit, Hmac};
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonWebToken {
    user_id: Uuid,
    scopes: Vec<String>,
    expires_at: NaiveDateTime,
}

impl JsonWebToken {
    pub fn new(user_id: Uuid, scopes: Vec<String>, expires_at: Option<NaiveDateTime>) -> Self {
        Self {
            user_id,
            scopes,
            expires_at: expires_at
                .unwrap_or_else(|| chrono::Utc::now().naive_utc() + chrono::Duration::hours(1)),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < chrono::Utc::now().naive_utc()
    }
}

use crate::{http::response::HttpResponse, state::application::ApplicationState};

pub async fn auth(
    State(state): State<Arc<ApplicationState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<HttpResponse>)> {
    let auth_header = match req.headers().get(header::AUTHORIZATION) {
        Some(header) => {
            let header = header.to_str().unwrap();
            let parts: Vec<&str> = header.split_whitespace().collect();
            if parts.len() == 2 {
                parts[1].to_string()
            } else {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(HttpResponse {
                        status: StatusCode::UNAUTHORIZED.as_u16(),
                        message: "Unauthorized".to_string(),
                        fields: None,
                    }),
                ));
            }
        }
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(HttpResponse {
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                    message: "Unauthorized".to_string(),
                    fields: None,
                }),
            ));
        }
    };

    let jwt_key: Hmac<Sha256> = Hmac::new_from_slice(state.jwt_key.as_bytes()).unwrap();
    let payload: JsonWebToken = match auth_header.verify_with_key(&jwt_key) {
        Ok(payload) => payload,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(HttpResponse {
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                    message: "Unauthorized".to_string(),
                    fields: None,
                }),
            ));
        }
    };

    if payload.is_expired() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(HttpResponse {
                status: StatusCode::UNAUTHORIZED.as_u16(),
                message: "Unauthorized".to_string(),
                fields: None,
            }),
        ));
    }

    let user_repository = UserRepository::new(state.db_pool.clone());
    match user_repository.find_by_id(&payload.user_id).await {
        Ok(user) => match user {
            Some(user) => {
                req.extensions_mut().insert(user);
                req.extensions_mut().insert(payload.scopes);
                Ok(next.run(req).await)
            }
            None => Err((
                StatusCode::UNAUTHORIZED,
                Json(HttpResponse {
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                    message: "Unauthorized".to_string(),
                    fields: None,
                }),
            )),
        },
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(HttpResponse {
                status: StatusCode::UNAUTHORIZED.as_u16(),
                message: "Unauthorized".to_string(),
                fields: None,
            }),
        )),
    }
}
