use axum::{
	extract::{State, Json},
	response::IntoResponse,
	routing::{get, post},
	Router,
};
use tower_cookies::{Cookies, Cookie};
use serde::Deserialize;

use axum_macros::debug_handler;

use crate::{state::AppState}; // adjust this if needed

use crate::routes::docs::get_user_id_from_cookie;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::StatusCode;

#[derive(Deserialize)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/login", post(login))
		.route("/logout", get(logout))
		.route("/me", get(me))
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
							Cookie::build("session", token)
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
		Ok(_) => (StatusCode::OK, "ok").into_response(),
		Err(_) => (StatusCode::UNAUTHORIZED, "not logged in").into_response(),
	}
}

pub async fn logout(cookies: Cookies) -> impl IntoResponse {
	cookies.remove(Cookie::named("session"));
	(StatusCode::OK, "logged out")
}
