use axum::{
	Json, Router,
	routing::{post, get, delete},
	response::{IntoResponse},
	extract::{State, Path},
	http::{StatusCode, HeaderMap}
};
use serde::Deserialize;

use crate::{state::AppState, users::UserManager};

#[derive(Deserialize)]
pub struct CreateUserPayload {
	pub username: String,
	pub password: String,
}

pub fn routes() -> Router<AppState> {
	Router::new()
		.route("/create", post(create_user))
		.route("/delete", delete(delete_user))
		.route("/update-password", post(update_password))
		.route("/exists/:username", get(user_exists))
		.route("/list", get(list_users))
}

async fn create_user(
	State(state): State<AppState>,
	headers: HeaderMap,
	Json(payload): Json<CreateUserPayload>,
) -> (StatusCode, &'static str) {
	if !is_authorized(&headers) {
		return (StatusCode::UNAUTHORIZED, "Unauthorized");
	}

	let manager = UserManager::new(state.db.clone());
	match manager.create_user(&payload.username, &payload.password).await {
		Ok(_) => (StatusCode::CREATED, "User created"),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user"),
	}
}

#[derive(Deserialize)]
pub struct DeleteUserPayload {
	pub username: String,
}

async fn delete_user(
	State(state): State<AppState>,
	headers: HeaderMap,
	Json(payload): Json<DeleteUserPayload>,
) -> (StatusCode, &'static str) {
	if !is_authorized(&headers) {
		return (StatusCode::UNAUTHORIZED, "Unauthorized");
	}

	let manager = UserManager::new(state.db.clone());
	match manager.delete_user(&payload.username).await {
		Ok(_) => (StatusCode::OK, "User deleted"),
		Err(_) => (StatusCode::NOT_FOUND, "User not found"),
	}
}

#[derive(Deserialize)]
pub struct UpdatePasswordPayload {
	pub username: String,
	pub password: String,
}

async fn update_password(
	State(state): State<AppState>,
	headers: HeaderMap,
	Json(payload): Json<UpdatePasswordPayload>,
) -> (StatusCode, &'static str) {
	if !is_authorized(&headers) {
		return (StatusCode::UNAUTHORIZED, "Unauthorized");
	}

	let manager = UserManager::new(state.db.clone());
	match manager.update_password(&payload.username, &payload.password).await {
		Ok(_) => (StatusCode::OK, "Password updated"),
		Err(_) => (StatusCode::NOT_FOUND, "User not found"),
	}
}

async fn user_exists(
	State(state): State<AppState>,
	headers: HeaderMap,
	Path(username): Path<String>,
) -> (StatusCode, &'static str) {
	if !is_authorized(&headers) {
		return (StatusCode::UNAUTHORIZED, "Unauthorized");
	}

	let manager = UserManager::new(state.db.clone());
	match manager.user_exists(&username).await {
		Ok(true) => (StatusCode::OK, "Exists"),
		Ok(false) => (StatusCode::NOT_FOUND, "Not found"),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error"),
	}
}

async fn list_users(
	State(state): State<AppState>,
	headers: HeaderMap,
) -> impl IntoResponse {
	if !is_authorized(&headers) {
		return (StatusCode::UNAUTHORIZED, Json(Vec::<String>::new()));
	}

	let manager = UserManager::new(state.db.clone());
	match manager.list_users().await {
		Ok(users) => (StatusCode::OK, Json(users)),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<String>::new())),
	}
}

fn is_authorized(headers: &HeaderMap) -> bool {
	match headers.get("Authorization").and_then(|v| v.to_str().ok()) {
		Some(value) if value == format!("Bearer {}", std::env::var("ADMIN_TOKEN").unwrap_or_default()) => true,
		_ => false,
	}
}
