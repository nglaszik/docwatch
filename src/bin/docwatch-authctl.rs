use std::{io::Write, fs::File};
use tiny_http::{Server, Response};

#[tokio::main]
async fn main() {
	
	dotenv::dotenv().ok();
	
	if std::env::var("ADMIN_TOKEN").is_err() {
		// Try fallback path (e.g., used in production)
		let _ = dotenv::from_path("/etc/docwatch/.env");
	}
	
	let client_id = std::env::var("GOOGLE_CLIENT_ID").expect("Missing GOOGLE_CLIENT_ID");
	let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").expect("Missing GOOGLE_CLIENT_SECRET");
	let redirect_uri = "http://localhost:8080/callback";
	let scope = "https://www.googleapis.com/auth/drive";

	let auth_url = format!(
		"https://accounts.google.com/o/oauth2/v2/auth?response_type=code&client_id={}&redirect_uri={}&scope={}&access_type=offline&prompt=consent",
		client_id,
		urlencoding::encode(redirect_uri),
		urlencoding::encode(scope)
	);
	
	if webbrowser::open(&auth_url).is_err() {
		println!("\nPlease open the following URL in your browser manually:\n\n{}\n", auth_url);
	}

	println!("Waiting for OAuth callback...");

	let server = Server::http("0.0.0.0:8080").unwrap();
	for request in server.incoming_requests() {
		let query = request.url();
		if query.starts_with("/callback?") {
			let code = query.split("code=").nth(1).unwrap().split('&').next().unwrap();

			let body = format!(
				"code={}&client_id={}&client_secret={}&redirect_uri={}&grant_type=authorization_code",
				code, client_id, client_secret, redirect_uri
			);

			let client = reqwest::Client::new();
			let res = client
				.post("https://oauth2.googleapis.com/token")
				.header("Content-Type", "application/x-www-form-urlencoded")
				.body(body)
				.send()
				.await
				.unwrap()
				.text()
				.await
				.unwrap();

			let mut file = File::create("/opt/docwatch/google_token.json").unwrap();
			file.write_all(res.as_bytes()).unwrap();

			request.respond(Response::from_string("Token saved. You can close this window.")).unwrap();
			break;
		}
	}

	println!("Token saved to google_token.json");
}
