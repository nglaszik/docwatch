use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::Serialize;

/// CLI for managing Docwatch users via the backend API
#[derive(Parser)]
#[command(name = "docwatch-userctl", about = "User management CLI for Docwatch")]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	Create {
		#[arg(short, long)]
		username: String,
		#[arg(short, long)]
		password: String,
	},
	Delete {
		#[arg(short, long)]
		username: String,
	},
	UpdatePassword {
		#[arg(short, long)]
		username: String,
		#[arg(short, long)]
		password: String,
	},
	Exists {
		#[arg(short, long)]
		username: String,
	},
	List,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv::dotenv().ok();
	
	if std::env::var("ADMIN_TOKEN").is_err() {
		// Try fallback path (e.g., used in production)
		let _ = dotenv::from_path("/etc/docwatch/.env");
	}

	let base_url = std::env::var("DOCWATCH_URL").unwrap_or_else(|_| "http://localhost:3009/docwatch/api".to_string());
	let token = std::env::var("ADMIN_TOKEN").expect("Missing ADMIN_TOKEN (set in .env)");

	let cli = Cli::parse();
	let client = Client::new();

	match cli.command {
		Commands::Create { username, password } => {
			let payload = CreateOrUpdate { username, password };
			let res = client
				.post(format!("{}/admin/create", base_url))
				.header("Authorization", format!("Bearer {}", token))
				.json(&payload)
				.send()
				.await?;

			if res.status().is_success() {
				println!("✅ User created");
			} else {
				println!("❌ Failed: {}", res.text().await?);
			}
		}

		Commands::Delete { username } => {
			let res = client
				.delete(format!("{}/admin/delete", base_url))
				.header("Authorization", format!("Bearer {}", token))
				.json(&serde_json::json!({ "username": username }))
				.send()
				.await?;

			if res.status().is_success() {
				println!("🗑️  User deleted");
			} else {
				println!("❌ Failed: {}", res.text().await?);
			}
		}

		Commands::UpdatePassword { username, password } => {
			let payload = CreateOrUpdate { username, password };
			let res = client
				.post(format!("{}/admin/update-password", base_url))
				.header("Authorization", format!("Bearer {}", token))
				.json(&payload)
				.send()
				.await?;

			if res.status().is_success() {
				println!("🔐 Password updated");
			} else {
				println!("❌ Failed: {}", res.text().await?);
			}
		}

		Commands::Exists { username } => {
			let res = client
				.get(format!("{}/admin/exists/{}", base_url, username))
				.header("Authorization", format!("Bearer {}", token))
				.send()
				.await?;

			if res.status().is_success() {
				println!("✅ User exists");
			} else if res.status() == reqwest::StatusCode::NOT_FOUND {
				println!("❌ User does not exist");
			} else {
				println!("❌ Error: {}", res.text().await?);
			}
		}

		Commands::List => {
			let res = client
				.get(format!("{}/admin/list", base_url))
				.header("Authorization", format!("Bearer {}", token))
				.send()
				.await?;

			if res.status().is_success() {
				let users: Vec<String> = res.json().await?;
				for user in users {
					println!("{}", user);
				}
			} else {
				println!("❌ Failed to list users: {}", res.text().await?);
			}
		}
	}

	Ok(())
}

#[derive(Serialize)]
struct CreateOrUpdate {
	username: String,
	password: String,
}
