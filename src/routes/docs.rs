use axum::{extract::{State, Json, Query}, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use crate::state::AppState;
use tower_cookies::{Cookies};
use axum::extract::Path;

use crate::poller::OwnedWordChange;
use crate::google_api::{add_docwatch_property}; //unused but keep for later in case useful

#[derive(Deserialize)]
pub struct DocQuery {
	q: Option<String>,
}

#[derive(Serialize)]
struct RevisionSummary {
	revision_time: String,
	added_words: Option<i64>,
	deleted_words: Option<i64>,
}

#[derive(sqlx::FromRow)]
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
	revision_summary: Vec<RevisionSummary>,
}

#[derive(Deserialize)]
pub struct AddDocRequest {
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
	Query(params): Query<DocQuery>,
) -> impl IntoResponse {
	match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(user_id) => {
			let q = params.q.as_deref().unwrap_or("").to_lowercase();

			// Fetch documents depending on search query
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

			// Enrich with revision summaries
			let mut docs_with_summaries = Vec::with_capacity(docs.len());

			for doc in docs {
				let revisions = sqlx::query_as!(
					RevisionSummary,
					r#"
					SELECT revision_time, added_words, deleted_words
					FROM document_revisions
					WHERE document_id = ?
					ORDER BY revision_time DESC
					LIMIT 100
					"#,
					doc.id
				)
				.fetch_all(&state.db)
				.await
				.unwrap_or_else(|_| vec![]);

				docs_with_summaries.push(DocInfo {
					doc_id: doc.doc_id,
					name: doc.name,
					last_updated: doc.last_updated,
					revision_summary: revisions,
					// Optionally add doc.owner_name here if you extend DocInfo
				});
			}

			Json(docs_with_summaries).into_response()
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

	(StatusCode::OK, "Document added").into_response()
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
		Ok(rows) => {
			let summaries = rows
				.into_iter()
				.map(|r| {
					let diff_str = r.diff.unwrap_or_else(|| "[]".to_string());

					// Try to parse the diff JSON
					let added_words = serde_json::from_str::<Vec<OwnedWordChange>>(&diff_str)
						.map(|changes| {
							changes
								.into_iter()
								.filter(|c| matches!(c, OwnedWordChange::Added(_)))
								.count()
						})
						.unwrap_or(0); // fallback if deserialization fails

					serde_json::json!({
						"revision_time": r.revision_time,
						"diff": diff_str,
						"added_words": added_words
					})
				})
				.collect::<Vec<_>>();

			Json(summaries).into_response()
		}
		Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch revisions").into_response(),
	}
}

