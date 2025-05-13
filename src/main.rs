use axum::{Router, Server};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;

mod routes;
mod state;
mod poller;
mod google_api;

pub mod users;

mod serve_static;
use serve_static::serve_spa;

use state::AppState;

use crate::poller::poll_loop;

#[tokio::main]
async fn main() {
	
	dotenv::dotenv().ok();
	
	println!("Docwatch is starting up!");
	
	let state = AppState::new().await;
	
	let db_pool_for_polling = state.db.clone();
	tokio::spawn(async move {
		poll_loop(db_pool_for_polling).await;
	});

	let app = Router::new()
		.nest("/docwatch/api", routes::routes())
		.layer(CookieManagerLayer::new())
		.with_state(state)
		.fallback(serve_spa);
		
	let port: u16 = std::env::var("PORT")
		.ok()
		.and_then(|s| s.parse().ok())
		.unwrap_or(3009);
	
	let addr = SocketAddr::from(([127, 0, 0, 1], port));
	println!("Docwatch server started on {}", addr);
	Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}