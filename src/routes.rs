use axum::Router;

mod auth;
mod docs;

use axum::routing::get;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
	Router::new()
		.nest("/auth", auth::routes())
		.route("/docs", get(docs::get_docs).post(docs::add_doc))
		.route("/docs/:doc_id/revisions", get(docs::get_revisions))
}

