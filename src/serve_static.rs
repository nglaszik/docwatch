// src/serve_static.rs
use axum::{
	body::Body,
	http::{StatusCode, Uri},
	response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};
use mime_guess::from_path;

static DIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

pub async fn serve_spa(uri: Uri) -> impl IntoResponse {
	let full_path = uri.path();
	let path = full_path
		.strip_prefix("/docwatch/")
		.or_else(|| full_path.strip_prefix("/docwatch"))
		.unwrap_or("")
		.trim_start_matches('/');

	// ✅ 1. Serve the file if it exists in embedded dir
	if let Some(file) = DIST_DIR.get_file(path) {
		let mime = from_path(path).first_or_octet_stream();
		return Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", mime.as_ref())
			.body(Body::from(file.contents()))
			.unwrap();
	}

	// ❌ 2. Do *not* serve index.html for asset paths — just return 404
	if path.contains('.') {
		return Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body(Body::from("Asset not found"))
			.unwrap();
	}

	// ✅ 3. Serve index.html for clean client-side routes (like /dashboard)
	if let Some(index) = DIST_DIR.get_file("index.html") {
		return Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", "text/html")
			.body(Body::from(index.contents()))
			.unwrap();
	}

	// Fail-safe: shouldn't reach here
	Response::builder()
		.status(StatusCode::INTERNAL_SERVER_ERROR)
		.body(Body::from("Missing index.html"))
		.unwrap()
}


