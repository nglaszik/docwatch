use axum::{Router, Server};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;

mod routes;
mod state;
mod poller;
mod google_api;

mod serve_static;
use serve_static::serve_spa;

use state::AppState;

use crate::poller::poll_loop;

#[tokio::main]
async fn main() {
	
	dotenv::dotenv().ok();
	
	let state = AppState::new().await;
	
	let db_pool_for_polling = state.db.clone();
	tokio::spawn(async move {
		poll_loop(db_pool_for_polling).await;
	});

	let app = Router::new()
		.nest("/api", routes::routes())
		.layer(CookieManagerLayer::new())
		.with_state(state)
		.fallback(serve_spa);


	let addr = SocketAddr::from(([127, 0, 0, 1], 3009));
	println!("🚀 Listening on {}", addr);
	Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}