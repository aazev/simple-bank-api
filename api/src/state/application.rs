use sqlx::PgPool;

#[allow(dead_code)]
pub struct ApplicationState {
    pub db_pool: PgPool,
    pub master_key: Vec<u8>,
}

impl ApplicationState {
    pub fn new(db_pool: PgPool, master_key: Vec<u8>) -> Self {
        Self {
            db_pool,
            master_key,
        }
    }
}
