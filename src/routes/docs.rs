use axum::{extract::{State, Json, Query}, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use tower_cookies::{Cookies};
use axum::extract::Path;

use crate::google_api::{add_docwatch_property}; //unused but keep for later in case useful
use crate::routes::auth::get_user_id_from_cookie;

use serde_json::json;

use uuid::Uuid;

#[derive(Deserialize)]
pub struct DocSearchQuery {
	q: Option<String>,
}

#[derive(Deserialize)]
pub struct EditPayload {
	id: String,
	field: String,
	value: String, // Represented as a string; can be parsed as needed
}

#[derive(Deserialize)]
pub struct CreateFolderPayload {
	name: Option<String>,
	id_parent: Option<String>,
}

#[derive(sqlx::FromRow, Serialize)]
struct DocRecord {
	id: i64,
	doc_id: String,
	name: String,
	last_updated: String,
	owner_username: String,
}

#[derive(sqlx::FromRow, Serialize)]
struct UserDocRecord {
	id: Option<String>,
	doc_id: Option<String>,
	name: Option<String>,
	is_folder: bool,
	last_updated: Option<String>,
	owner_username: Option<String>
}

#[derive(Serialize)]
struct Breadcrumb {
	id: Option<String>,
	name: String,
}

#[derive(Serialize)]
struct UserDocResponse {
	breadcrumbs: Vec<Breadcrumb>,
	docs: Vec<UserDocRecord>,
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
	Query(params): Query<DocSearchQuery>,
) -> impl IntoResponse {
	match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(_user_id) => {
			let q = params.q.as_deref().unwrap_or("").to_lowercase();
			let docs: Vec<DocRecord> = {
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
			};
			Json(docs).into_response()
		}
		Err(code) => (code, "Unauthorized").into_response(),
	}
}

pub async fn get_user_docs(
	State(state): State<AppState>,
	cookies: Cookies,
	Path(id_parent): Path<String>,
) -> impl IntoResponse {
	let user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(id) => id,
		Err(code) => return (code, "Unauthorized").into_response(),
	};

	let mut breadcrumbs: Vec<Breadcrumb> = vec![];	
	let is_home = id_parent == "home";
	let mut current_id = if is_home { None } else { Some(id_parent.clone()) };
	let parent_id = if is_home { None } else { Some(id_parent) };
	
	while let Some(id) = current_id {
		if let Ok(Some(row)) = sqlx::query!(
			r#"
			SELECT id, folder_name, id_parent
			FROM user_documents
			WHERE id = ? AND user_id = ? AND is_folder = TRUE
			"#,
			id,
			user_id
		)
		.fetch_optional(&state.db)
		.await
		{
			breadcrumbs.push(Breadcrumb {
				id: row.id,
				name: row.folder_name.unwrap_or_else(|| "Untitled Folder".to_string()),
			});
			current_id = row.id_parent;
		} else {
			break;
		}
	}

	breadcrumbs.reverse(); // So root is first, current folder last

	let docs: Vec<UserDocRecord> = if let Some(parent_id) = parent_id {
		sqlx::query_as!(
			UserDocRecord,
			r#"
			SELECT
				ud.id,
				d.doc_id,
				COALESCE(ud.folder_name, d.name) AS name,
				ud.is_folder,
				d.last_updated,
				d.owner_username
			FROM user_documents ud
			LEFT JOIN documents d ON ud.document_id = d.id
			WHERE ud.user_id = ? AND ud.id_parent = ?
			"#,
			user_id,
			parent_id
		)
		.fetch_all(&state.db)
		.await
		.unwrap_or_default()
	} else {
		sqlx::query_as!(
			UserDocRecord,
			r#"
			SELECT
				ud.id,
				d.doc_id,
				COALESCE(ud.folder_name, d.name) AS name,
				ud.is_folder,
				d.last_updated,
				d.owner_username
			FROM user_documents ud
			LEFT JOIN documents d ON ud.document_id = d.id
			WHERE ud.user_id = ? AND ud.id_parent IS NULL
			"#,
			user_id
		)
		.fetch_all(&state.db)
		.await
		.unwrap_or_default()
	};

	Json(UserDocResponse {
		breadcrumbs,
		docs,
	})
	.into_response()
}

pub async fn create_folder(
	State(state): State<AppState>,
	cookies: Cookies,
	Json(payload): Json<CreateFolderPayload>,
) -> impl IntoResponse {
	
	let user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(id) => id,
		Err(code) => return (code, "Unauthorized").into_response(),
	};
	
	let new_id = Uuid::new_v4().to_string();

	let folder_name = payload.name.unwrap_or_else(|| "Untitled Folder".to_string());
	
	let id_parent = match payload.id_parent.as_deref() {
		Some("home") => None,
		other => other.map(|s| s.to_string()),
	};

	let id = match sqlx::query_scalar!(
		r#"
		INSERT INTO user_documents (id, user_id, is_folder, folder_name, id_parent)
		VALUES (?, ?, TRUE, ?, ?)
		RETURNING id
		"#,
		new_id,
		user_id,
		folder_name,
		id_parent
	)
	.fetch_one(&state.db)
	.await
	{
		Ok(id) => id,
		Err(e) => {
			eprintln!("Error creating folder: {:?}", e);
			return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create folder").into_response();
		}
	};

	Json(json!({ "id": id })).into_response()
}

pub async fn edit_user_documents(
	State(state): State<AppState>,
	cookies: Cookies,
	Json(payload): Json<EditPayload>,
) -> impl IntoResponse {
	let user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(id) => id,
		Err(code) => return (code, "Unauthorized").into_response(),
	};

	let result = match payload.field.as_str() {
		"id_parent" => {
			let new_parent = match payload.value.as_str() {
				"home" => None,
				other => Some(other.to_string()),
			};
			sqlx::query!(
				r#"
				UPDATE user_documents
				SET id_parent = ?
				WHERE id = ? AND user_id = ?
				"#,
				new_parent,
				payload.id,
				user_id
			)
			.execute(&state.db)
			.await
		}
		"folder_name" => {
			sqlx::query!(
				r#"
				UPDATE user_documents
				SET folder_name = ?
				WHERE id = ? AND user_id = ? AND is_folder = TRUE
				"#,
				payload.value,
				payload.id,
				user_id
			)
			.execute(&state.db)
			.await
		}
		_ => return (StatusCode::BAD_REQUEST, "Unsupported field").into_response(),
	};

	if result.is_err() {
		return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update document").into_response();
	}

	(StatusCode::OK, "OK").into_response()
}

// this adds a document to a user's watchlist
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
	cookies: Cookies,
	Path(rev_id): Path<i64>,
) -> impl IntoResponse {
	
	let _user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(id) => id,
		Err(code) => return (code, "Unauthorized").into_response(),
	};
	
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

pub async fn delete_user_doc(
	State(state): State<AppState>,
	cookies: Cookies,
	Path(id): Path<String>,
) -> impl IntoResponse {
	let user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(id) => id,
		Err(code) => return (code, "Unauthorized").into_response(),
	};

	if let Err(_) = sqlx::query!(
		r#"
		DELETE FROM user_documents
		WHERE id_parent = ? AND user_id = ?
		"#,
		id,
		user_id
	)
	.execute(&state.db)
	.await
	{
		return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete children").into_response();
	}

	if let Err(_) = sqlx::query!(
		r#"
		DELETE FROM user_documents
		WHERE id = ? AND user_id = ?
		"#,
		id,
		user_id
	)
	.execute(&state.db)
	.await
	{
		return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete document").into_response();
	}

	(StatusCode::NO_CONTENT, "").into_response()
}

pub async fn get_revisions(
	State(state): State<AppState>,
	cookies: Cookies,
	Path(doc_id): Path<String>,
) -> impl IntoResponse {
	
	let _user_id = match get_user_id_from_cookie(&state.db, &cookies).await {
		Ok(id) => id,
		Err(code) => return (code, "Unauthorized").into_response(),
	};
	
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

