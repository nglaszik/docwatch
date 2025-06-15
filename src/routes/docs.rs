use axum::{extract::{State, Json, Query}, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use tower_cookies::{Cookies};
use axum::extract::Path;

use crate::poller::OwnedWordChange;
use crate::google_api::{add_docwatch_property}; //unused but keep for later in case useful
use crate::routes::auth::get_user_id_from_cookie;

#[derive(Deserialize)]
pub struct DocQuery {
	q: Option<String>,
}

#[derive(sqlx::FromRow, Serialize)]
struct DocRecord {
	id: i64,
	doc_id: String,
	name: String,
	last_updated: String,
	owner_username: String,
}

#[derive(Serialize)]
struct DocInfo {
	doc_id: String,
	name: String,
	last_updated: String,
	owner_username: String,
}

#[derive(Deserialize)]
struct RawWordChange {
	#[serde(rename = "type")]
	change_type: String,
	text: String,
}

#[derive(Serialize)]
struct DiffBlock {
	#[serde(rename = "type")]
	block_type: String, // "add", "del", or "neutral"
	text: String,
}

#[derive(Deserialize)]
pub struct AddDocRequest {
	doc_id: String,
}

pub async fn get_docs(
	State(state): State<AppState>,
	cookies: Cookies,
	Query(params): Query<DocQuery>,
) -> impl IntoResponse {
	match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(user_id) => {
			let q = params.q.as_deref().unwrap_or("").to_lowercase();

			// Fetch documents depending on search query
			// We want to 1. display last_updated time
			// 2. add functionality where we return a list of the latest updated documents
			
			let docs: Vec<DocRecord> = if !q.is_empty() {
				let wildcard = format!("%{}%", q);
				sqlx::query_as!(
					DocRecord,
					r#"
					SELECT d.id, d.doc_id, d.name, d.last_updated, d.owner_username
					FROM documents d
					WHERE LOWER(d.doc_id) LIKE ? OR LOWER(d.name) LIKE ? OR LOWER(d.owner_username) LIKE ?
					ORDER BY d.last_updated DESC
					LIMIT 20
					"#,
					wildcard,
					wildcard,
					wildcard
				)
				.fetch_all(&state.db)
				.await
				.unwrap_or_else(|_| vec![])
			} else {
				// return all documents that the user is watching
				sqlx::query_as!(
					DocRecord,
					r#"
					SELECT d.id, d.doc_id, d.name, d.last_updated, d.owner_username
					FROM user_documents ud
					JOIN documents d ON ud.document_id = d.id
					WHERE ud.user_id = ?
					"#,
					user_id
				)
				.fetch_all(&state.db)
				.await
				.unwrap_or_else(|_| vec![])
			};
			Json(docs).into_response()
		}
		Err(code) => (code, "Unauthorized").into_response(),
	}
}

pub async fn add_doc(
	State(state): State<AppState>,
	cookies: Cookies,
	Json(payload): Json<AddDocRequest>,
) -> impl IntoResponse {
	let user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(uid) => uid,
		Err(code) => return (code, "Unauthorized").into_response(),
	};

	// Look up internal document ID
	let doc = sqlx::query!(
		"SELECT id FROM documents WHERE doc_id = ?",
		payload.doc_id
	)
	.fetch_optional(&state.db)
	.await;

	let doc_id = match doc {
		Ok(Some(row)) => row.id,
		_ => return (StatusCode::NOT_FOUND, "Document not found").into_response(),
	};

	// Add to user's watchlist (idempotent insert)
	let _ = sqlx::query!(
		"INSERT OR IGNORE INTO user_documents (user_id, document_id) VALUES (?, ?)",
		user_id,
		doc_id
	)
	.execute(&state.db)
	.await;

	StatusCode::NO_CONTENT.into_response()
}

pub async fn get_diff(
	State(state): State<AppState>,
	Path(rev_id): Path<i64>,
) -> impl IntoResponse {
	let result = sqlx::query!(
		r#"
		SELECT diff FROM document_revisions WHERE id = ?
		"#,
		rev_id
	)
	.fetch_optional(&state.db)
	.await;

	match result {
		Ok(Some(row)) => {
			let diff_str = row.diff.unwrap_or_else(|| "[]".to_string());

			let parsed: Result<Vec<RawWordChange>, _> = serde_json::from_str(&diff_str);
			let transformed = match parsed {
				Ok(items) => {
					let result: Vec<DiffBlock> = items
						.into_iter()
						.map(|item| {
							let block_type = match item.change_type.as_str() {
								"Added" => "add",
								"Removed" => "del",
								"Unchanged" => "neutral",
								_ => "neutral",
							};
							DiffBlock {
								block_type: block_type.to_string(),
								text: item.text,
							}
						})
						.collect();
					Json(result).into_response()
				}
				Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid diff format").into_response(),
			};
			transformed
		}
		Ok(None) => (StatusCode::NOT_FOUND, "Revision not found").into_response(),
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
	}
}

pub async fn get_revisions(
	State(state): State<AppState>,
	Path(doc_id): Path<String>,
) -> impl IntoResponse {
	let revisions = sqlx::query!(
		r#"
		SELECT r.id, r.revision_time, r.added_words, r.deleted_words
		FROM document_revisions r
		JOIN documents d ON r.document_id = d.id
		WHERE d.doc_id = ?
		ORDER BY r.revision_time DESC
		"#,
		doc_id
	)
	.fetch_all(&state.db)
	.await;

	match revisions {
		Ok(rows) => {
			let summaries = rows
				.into_iter()
				.map(|r| {
					serde_json::json!({
						"id": r.id,
						"revision_time": r.revision_time,
						"added_words": r.added_words.unwrap_or(0),
						"deleted_words": r.deleted_words.unwrap_or(0),
					})
				})
				.collect::<Vec<_>>();

			Json(summaries).into_response()
		}
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch revisions").into_response(),
	}
}

