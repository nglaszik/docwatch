use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
	pub db: SqlitePool,
}

impl AppState {
	pub async fn new() -> Self {
		// Load the DATABASE_URL from the environment (.env should already be loaded by main.rs)
		let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

		// Connect to the database
		let db = SqlitePool::connect(&database_url)
			.await
			.expect("Failed to connect to database");

		// Run migrations to create tables if they don't exist
		sqlx::migrate!("./migrations")
			.run(&db)
			.await
			.expect("Failed to run database migrations");

		Self { db }
	}
}