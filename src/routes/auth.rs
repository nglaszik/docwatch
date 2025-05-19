use axum::{
	extract::{State, Json},
	response::IntoResponse,
	routing::{get, post},
	Router,
};
use tower_cookies::{Cookies, Cookie};
use serde::Deserialize;
use serde::Serialize;

use axum_macros::debug_handler;

use crate::{state::AppState};

use sqlx::SqlitePool;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::StatusCode;

#[derive(Deserialize)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,
}

#[derive(Serialize)]
struct MeResponse {
	username: String,
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/login", post(login))
		.route("/logout", get(logout))
		.route("/me", get(me))
}

pub async fn get_user_id_from_cookie(
	db: &SqlitePool,
	cookies: &Cookies,
) -> Result<i64, StatusCode> {
	if let Some(cookie) = cookies.get("docwatch-session-id") {
		let token = cookie.value();
		let user_id = sqlx::query_scalar!(
			"SELECT user_id FROM sessions WHERE token = ?",
			token
		)
		.fetch_optional(db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

		if let Some(uid) = user_id {
			return Ok(uid);
		}
	}

	Err(StatusCode::UNAUTHORIZED)
}


pub async fn login(
	State(state): State<AppState>,
	cookies: Cookies,
	Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
	let user = sqlx::query!(
		"SELECT id, password FROM users WHERE username = ?",
		payload.username
	)
	.fetch_optional(&state.db)
	.await;

	match user {
		Ok(Some(user)) => {
			if let Ok(parsed_hash) = PasswordHash::new(&user.password) {
				let verified = Argon2::default()
					.verify_password(payload.password.as_bytes(), &parsed_hash)
					.is_ok();

				if verified {
					let token = uuid::Uuid::new_v4().to_string();
					let result = sqlx::query!(
						"INSERT INTO sessions (user_id, token) VALUES (?, ?)",
						user.id,
						token
					)
					.execute(&state.db)
					.await;

					if result.is_ok() {
						cookies.add(
							Cookie::build("docwatch-session-id", token)
								.path("/")
								.http_only(true)
								.finish(),
						);
						return "ok".into_response();
					} else {
						return (
							StatusCode::INTERNAL_SERVER_ERROR,
							"Failed to create session",
						)
							.into_response();
					}
				}
			}
		}
		Ok(None) | Err(_) => {}
	}

	(StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
}

pub async fn me(
	State(state): State<AppState>,
	cookies: Cookies,
) -> impl IntoResponse {
	match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(user_id) => {
			let user = sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
				.fetch_optional(&state.db)
				.await;

			match user {
				Ok(Some(user)) => axum::Json(MeResponse { username: user.username }).into_response(),
				_ => (StatusCode::INTERNAL_SERVER_ERROR, "User not found").into_response(),
			}
		}
		Err(_) => (StatusCode::UNAUTHORIZED, "not logged in").into_response(),
	}
}

pub async fn logout(
	State(state): State<AppState>,
	cookies: Cookies,
) -> impl IntoResponse {
	if let Some(cookie) = cookies.get("docwatch-session-id") {
		let token = cookie.value();

		// Delete session from DB
		let _ = sqlx::query!(
			"DELETE FROM sessions WHERE token = ?",
			token
		)
		.execute(&state.db)
		.await;
	}
		
	// expire the cookie in browser
	let mut cookie = Cookie::named("docwatch-session-id");
	cookie.make_removal();
	cookies.remove(cookie);

	(StatusCode::OK, "logged out")
}

