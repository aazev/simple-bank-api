use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::{Duration, Local};
use database::{
    filters::user::Filter as UserFilter, services::user::Service as UserService, verify_password,
};
use hmac::Hmac;
use jwt::SignWithKey;
use sha2::{digest::KeyInit, Sha256};

use crate::{
    http::{
        auth::{AuthRequest, AuthResponse},
        response::HttpResponse,
    },
    middlewares::auth::JsonWebToken,
    state::application::ApplicationState,
};

pub fn get_router() -> Router<Arc<ApplicationState>> {
    Router::new().route("/auth", post(authorize_user))
}

#[utoipa::path(
    post,
    path = "/auth",
    context_path = "/api/v1",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Successful authorization response", body = AuthResponse),
        (status = 401, description = "Unauthorized", body = HttpResponse, example = json!(r#"{"status": 401, "message": "Invalid credentials"}"#)),
        (status = 403, description = "Forbidden", body = HttpResponse, example = json!(r#"{"status": 403, "message": "Invalid credentials"}"#)),
        (status = 500, description = "Internal Server Error", body = HttpResponse, example = json!(r#"{"status": 500, "message": "Internal Server Error"}"#))
    ),
)]
pub async fn authorize_user(
    State(state): State<Arc<ApplicationState>>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<HttpResponse>)> {
    let filter: UserFilter = UserFilter {
        email: Some(payload.email),
        ..Default::default()
    };

    let user_service = UserService::new();

    match user_service
        .get_one_by_filter(&state.db_pool, &filter)
        .await
    {
        Some(user) => match verify_password(&user.password, &payload.password) {
            Ok(_) => {
                let token_creation = Local::now().naive_utc();
                let token_expiration = token_creation + Duration::hours(1);
                let jwt_key: Hmac<Sha256> = Hmac::new_from_slice(state.jwt_key.as_bytes()).unwrap();
                let token = JsonWebToken::new(user.id, payload.scopes, Some(token_expiration));

                match token.sign_with_key(&jwt_key) {
                    Ok(token) => {
                        let auth_token = AuthResponse::new(token);
                        Ok(Json(auth_token))
                    }
                    Err(_) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(HttpResponse {
                            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                            message: "Failed to create token".to_string(),
                            fields: None,
                        }),
                    )),
                }
            }
            Err(_) => Err((
                StatusCode::FORBIDDEN,
                Json(HttpResponse {
                    status: StatusCode::FORBIDDEN.as_u16(),
                    message: "Invalid credentials".to_string(),
                    fields: None,
                }),
            )),
        },
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(HttpResponse {
                status: StatusCode::UNAUTHORIZED.as_u16(),
                message: "Unauthorized".to_string(),
                fields: None,
            }),
        )),
    }
}
