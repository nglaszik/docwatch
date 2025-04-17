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
	let path = uri.path().trim_start_matches('/');

	if let Some(file) = DIST_DIR.get_file(path) {
		let mime = from_path(path).first_or_octet_stream();
		return Response::builder()
			.status(StatusCode::OK)
			.header("Content-Type", mime.as_ref())
			.body(Body::from(file.contents()))
			.unwrap();
	}

	let index = DIST_DIR.get_file("index.html").unwrap();
	Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "text/html")
		.body(Body::from(index.contents()))
		.unwrap()
}
