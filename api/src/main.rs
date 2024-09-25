use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    http::{
        header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE, REFERER},
        HeaderName, HeaderValue, Method, StatusCode,
    },
    middleware as axum_middleware,
    response::IntoResponse,
    Json, Router,
};
use database::{get_database_pool, load_master_key};
use dotenv::dotenv;
use http::response::HttpResponse;
use middlewares::auth::auth;
use routers::{
    accounts::get_router as get_accounts_router, auth::get_router as get_auth_router,
    transactions::get_router as get_transactions_router, users::get_router as get_users_router,
};
use state::application::ApplicationState;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tokio::{net::TcpListener, signal::ctrl_c};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{Any, CorsLayer};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub mod http;
mod middlewares;
mod routers;
mod state;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let mut num_cpus = num_cpus::get() as u32;
    let dedicated: bool = std::env::var("DEDICATED_SERVER")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap();

    if !dedicated {
        println!("This is not a dedicated server.");
        // ensure we use N-2 threads for the runtime, or N-1 if N < 3
        num_cpus = if num_cpus < 3 {
            num_cpus - 1
        } else {
            num_cpus - 2
        };
    } else {
        println!("This is a dedicated server.");
    }

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus as usize)
        .enable_all()
        .build()?
        .block_on(async_main(num_cpus as usize));

    Ok(())
}

async fn async_main(threads: usize) {
    let min: Option<u32> = std::env::var("MIN_CONNECTIONS")
        .unwrap_or(threads.to_string())
        .parse::<u32>()
        .map(Some)
        .map_err(|_| None::<u32>)
        .unwrap();

    let max: Option<u32> = std::env::var("MAX_CONNECTIONS")
        .unwrap_or((threads * 2).to_string())
        .parse::<u32>()
        .map(Some)
        .map_err(|_| None::<u32>)
        .unwrap();

    let db_pool = get_database_pool(min, max).await;
    let master_key: Vec<u8> = load_master_key().expect("Failed to load master key");
    let jwt_key = std::env::var("JWT_KEY")
        .unwrap_or("eaccbdc5-dd87-40dc-a998-6a6fa26a5fa5.simple_bank_api".to_string());

    let app_state = Arc::new(ApplicationState::new(db_pool, master_key, jwt_key));

    let user_router = get_users_router();
    let accounts_router = get_accounts_router();
    let transactions_router = get_transactions_router();
    let auth_router = get_auth_router();

    let protected_routers = Router::new()
        .merge(user_router)
        .merge(accounts_router)
        .merge(transactions_router)
        .layer(axum_middleware::from_fn_with_state(app_state.clone(), auth));

    let unprotected_routers = Router::new().merge(auth_router);

    let api_base = Router::new()
        .merge(protected_routers)
        .merge(unprotected_routers)
        .with_state(app_state.clone());

    // initialize routers
    let api = Router::new()
        .nest("/api/v1", api_base)
        .fallback(deal_with_it);

    let handle = tokio::spawn(address_serve(api, threads));

    ctrl_c().await.expect("Failed to install CTRL-C handler");

    handle.abort();
}

async fn deal_with_it() -> (StatusCode, Json<HttpResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(HttpResponse::new(404, "Not found".to_string(), None)),
    )
}

async fn address_serve(rt: Router, threads: usize) -> impl IntoResponse {
    let concurrency_limit_layer: ConcurrencyLimitLayer = ConcurrencyLimitLayer::new(25);
    let allowed_origins: Option<Vec<HeaderValue>> = env::var("CORS_ORIGINS").ok().map(|s| {
        s.split(',')
            .map(|s| s.to_string().parse::<HeaderValue>().unwrap())
            .collect()
    });
    let cors = match &allowed_origins {
        Some(allowed_origins) => CorsLayer::new()
            .allow_origin(allowed_origins.to_vec())
            .allow_methods([
                Method::GET,
                Method::OPTIONS,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([
                AUTHORIZATION,
                ACCEPT,
                CONTENT_TYPE,
                ACCESS_CONTROL_ALLOW_ORIGIN,
                REFERER,
                HeaderName::from_static("api_scopes"),
            ]),
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::OPTIONS,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([
                AUTHORIZATION,
                ACCEPT,
                CONTENT_TYPE,
                ACCESS_CONTROL_ALLOW_ORIGIN,
                REFERER,
                HeaderName::from_static("api_scopes"),
            ]),
    };
    let address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS must be set.");
    let server_address: SocketAddr = address
        .parse::<SocketAddr>()
        .expect("Failed to parse server address.");

    println!(
        "Starting api on address: {} with {} threads...",
        &address, threads
    );

    let listener = TcpListener::bind(&server_address)
        .await
        .expect("Failed to bind to server address.");

    axum::serve(listener, rt.layer(concurrency_limit_layer).layer(cors))
        .await
        .unwrap()
}
