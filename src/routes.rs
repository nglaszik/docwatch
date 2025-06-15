use axum::Router;

mod auth;
mod docs;
mod admin;

use axum::routing::get;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
	Router::new()
		.nest("/auth", auth::routes())
		.nest("/admin", admin::routes())
		.route("/docs", get(docs::get_docs).post(docs::add_doc))
		.route("/docs/:doc_id/revisions", get(docs::get_revisions))
		.route("/diffs/:rev_id", get(docs::get_diff))
}

