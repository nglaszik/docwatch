use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
	pub db: SqlitePool,
}

impl AppState {
	pub async fn new() -> Self {
		let db = SqlitePool::connect("sqlite://data/docwatch.db").await.unwrap();
		Self { db }
	}
}