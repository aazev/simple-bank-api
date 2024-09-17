pub mod models;
pub mod structs;
pub mod traits;

use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn get_database_pool(min: Option<u32>, max: Option<u32>) -> PgPool {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let dedicated: bool = std::env::var("DEDICATED_SERVER")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap();
    let cpus = num_cpus::get() as u32;
    // if dedicated then use all cpus, if not, use all but 2 with the minimum being 1
    let min_connections = match dedicated {
        true => cpus,
        // not a dedicated server, so leave 2 cpus for the OS, but keep cpus with a minimum of 1
        false => cpus.saturating_sub(2).max(1),
    };
    let max_connections = cpus * 2;

    PgPoolOptions::new()
        .max_connections(max.unwrap_or(max_connections))
        .min_connections(min.unwrap_or(min_connections))
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}
