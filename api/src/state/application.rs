use sqlx::PgPool;

pub struct ApplicationState {
    pub db_pool: PgPool,
    pub master_key: Vec<u8>,
}
