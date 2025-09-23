use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub async fn create_pool() -> ConnectionPool {
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost port=5432 user=postgres password=9155 dbname=tunisian_citizens",
        NoTls,
    )
        .expect("Invalid connection string");

    Pool::builder()
        .max_size(10)
        .build(manager)
        .await
        .expect("Failed to build pool")
}

pub async fn init_db(pool: &ConnectionPool) {
    let conn = pool.get().await.expect("Failed to get connection");
    conn.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS api_usage (
            id SERIAL PRIMARY KEY,
            user_id TEXT NOT NULL,
            api_link TEXT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
    ",
    )
        .await
        .expect("Failed to create tables");
}
