// tests/database_tests.rs
use calibre_web_rust::infrastructure::database::create_postgres_pool;
use sqlx::PgPool;

#[tokio::test]
async fn test_create_postgres_pool() {
    // Use environment variable for test database
    let db_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/test".to_string());

    let pool = create_postgres_pool(&db_url, 5).await.unwrap();
    assert_eq!(pool.size(), 5);

    // Verify connection works
    let result: Option<(i64,)> = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(result, Some((1,)));
}
