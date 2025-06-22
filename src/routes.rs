use axum::Router;

mod auth;
mod docs;
mod admin;

use axum::routing::get;
use axum::routing::post;
use axum::routing::delete;
use axum::routing::patch;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
	Router::new()
		.nest("/auth", auth::routes())
		.nest("/admin", admin::routes())
		.route("/docs", get(docs::get_docs).post(docs::add_doc))
		.route("/docs/:doc_id/revisions", get(docs::get_revisions))
		.route("/diffs/:rev_id", get(docs::get_diff))
		
		// Folder & watchlist organization
		.route("/user_documents/create_folder", post(docs::create_folder))
		.route("/user_documents/edit", patch(docs::edit_user_documents))
		.route("/user_documents/:user_doc_id", get(docs::get_user_docs))
		.route("/user_documents/:user_doc_id", delete(docs::delete_user_doc))
}

