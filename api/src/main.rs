use std::{env, net::SocketAddr};

use axum::{
    http::{
        header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION, CONTENT_TYPE, REFERER},
        HeaderName, HeaderValue, Method, StatusCode,
    },
    response::IntoResponse,
    Json, Router,
};
use dotenv::dotenv;
use http::response::HttpResponse;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tokio::{net::TcpListener, signal::ctrl_c};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{Any, CorsLayer};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub mod http;
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
    // initialize routers

    let api = Router::new().fallback(deal_with_it);

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
