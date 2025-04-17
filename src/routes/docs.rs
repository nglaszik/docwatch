use axum::{extract::{State, Json}, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::state::AppState;
use tower_cookies::{Cookies, Cookie};
use axum::extract::Path;

use crate::google_api::{get_doc_info, add_docwatch_property};

struct CurrentUser {
	id: i64,
}

#[derive(Serialize)]
struct DocInfo {
	doc_id: String,
	name: String,
	last_updated: String,
}

#[derive(Deserialize)]
pub struct NewDoc {
	doc_id: String,
}

pub async fn get_user_id_from_cookie(
	db: &SqlitePool,
	cookies: &Cookies,
) -> Result<i64, StatusCode> {
	if let Some(cookie) = cookies.get("session") {
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

pub async fn get_docs(
	State(state): State<AppState>,
	cookies: Cookies,
) -> impl IntoResponse {
	match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(user_id) => {
			let docs = sqlx::query_as!(
				DocInfo,
				"SELECT doc_id, name, last_updated FROM documents WHERE user_id = ?",
				user_id
			)
			.fetch_all(&state.db)
			.await
			.unwrap_or_else(|_| vec![]);

			Json(docs).into_response()
		}
		Err(code) => (code, "Unauthorized").into_response(),
	}
}

pub async fn add_doc(
	State(state): State<AppState>,
	cookies: Cookies,
	Json(payload): Json<NewDoc>,
) -> impl IntoResponse {
	match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(user_id) => {
			// Step 0: Check if doc already exists
			let exists = sqlx::query!("SELECT id FROM documents WHERE doc_id = ?", payload.doc_id)
				.fetch_optional(&state.db)
				.await
				.unwrap_or(None);  // ✅ use unwrap_or to avoid `?`

			if exists.is_some() {
				return (StatusCode::CONFLICT, "Document already added").into_response();
			}

			// Step 1: Fetch name + modifiedTime in one call
			if let Ok((name, modified_time)) = get_doc_info(&payload.doc_id).await {
				// Step 2: Insert into DB
				let _ = sqlx::query("INSERT INTO documents (user_id, doc_id, name, last_updated) VALUES (?, ?, ?, ?)")
					.bind(user_id)
					.bind(&payload.doc_id)
					.bind(name)
					.bind(modified_time)
					.execute(&state.db)
					.await;

				// Step 3: Add docwatch tag
				if let Err(e) = add_docwatch_property(&payload.doc_id).await {
					eprintln!("⚠️ Failed to tag file: {:?}", e);
				}

				"Added".into_response()
			} else {
				(StatusCode::BAD_REQUEST, "Document not accessible").into_response()
			}
		}
		Err(code) => (code, "Unauthorized").into_response(),
	}
}

pub async fn get_revisions(
	State(state): State<AppState>,
	Path(doc_id): Path<String>,
) -> impl IntoResponse {
	let revisions = sqlx::query!(
		"SELECT r.revision_time, r.diff
		 FROM document_revisions r
		 JOIN documents d ON r.document_id = d.id
		 WHERE d.doc_id = ?
		 ORDER BY r.revision_time DESC",
		doc_id
	)
	.fetch_all(&state.db)
	.await;

	match revisions {
		Ok(rows) => Json(
			rows.into_iter()
				.map(|r| {
					let added_chars = r.diff
						.as_deref()             // converts Option<String> → Option<&str>
						.unwrap_or("")          // safely handle None
						.lines()
						.filter(|l| l.starts_with('+'))
						.map(|l| l.len())
						.sum::<usize>();
					serde_json::json!({
						"revision_time": r.revision_time,
						"diff": r.diff,
						"added_chars": added_chars
					})
				})
				.collect::<Vec<_>>()
		).into_response(),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch revisions").into_response(),
	}
}
