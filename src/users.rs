use sqlx::{SqlitePool, Row};
use argon2::{Argon2, PasswordHasher};
use password_hash::SaltString;
use rand_core::OsRng;

pub struct UserManager {
	pool: SqlitePool,
}

impl UserManager {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}

	/// Create a new user with hashed password
	pub async fn create_user(&self, username: &str, password: &str) -> Result<(), String> {
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = Argon2::default()
			.hash_password(password.as_bytes(), &salt)
			.map_err(|e| format!("Hashing failed: {}", e))?
			.to_string();

		sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
			.bind(username)
			.bind(password_hash)
			.execute(&self.pool)
			.await
			.map_err(|e| format!("DB error: {}", e))?;

		Ok(())
	}

	/// Delete a user by username
	pub async fn delete_user(&self, username: &str) -> Result<(), String> {
		let rows = sqlx::query("DELETE FROM users WHERE username = ?")
			.bind(username)
			.execute(&self.pool)
			.await
			.map_err(|e| format!("DB error: {}", e))?;

		if rows.rows_affected() == 0 {
			Err("User not found".to_string())
		} else {
			Ok(())
		}
	}

	/// Update a user's password
	pub async fn update_password(&self, username: &str, new_password: &str) -> Result<(), String> {
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = Argon2::default()
			.hash_password(new_password.as_bytes(), &salt)
			.map_err(|e| format!("Hashing failed: {}", e))?
			.to_string();

		let rows = sqlx::query("UPDATE users SET password = ? WHERE username = ?")
			.bind(password_hash)
			.bind(username)
			.execute(&self.pool)
			.await
			.map_err(|e| format!("DB error: {}", e))?;

		if rows.rows_affected() == 0 {
			Err("User not found".to_string())
		} else {
			Ok(())
		}
	}

	/// Check if a user exists
	pub async fn user_exists(&self, username: &str) -> Result<bool, String> {
		let row = sqlx::query("SELECT 1 FROM users WHERE username = ? LIMIT 1")
			.bind(username)
			.fetch_optional(&self.pool)
			.await
			.map_err(|e| format!("DB error: {}", e))?;

		Ok(row.is_some())
	}

	/// List all usernames
	pub async fn list_users(&self) -> Result<Vec<String>, String> {
		let rows = sqlx::query("SELECT username FROM users ORDER BY username")
			.fetch_all(&self.pool)
			.await
			.map_err(|e| format!("DB error: {}", e))?;

		Ok(rows.iter().map(|r| r.get::<String, _>("username")).collect())
	}
}
