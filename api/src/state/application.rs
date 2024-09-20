use sqlx::PgPool;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ApplicationState {
    pub db_pool: PgPool,
    pub master_key: Vec<u8>,
    pub jwt_key: String,
}

impl ApplicationState {
    pub fn new(db_pool: PgPool, master_key: Vec<u8>, jwt_key: String) -> Self {
        Self {
            db_pool,
            master_key,
            jwt_key,
        }
    }
}
