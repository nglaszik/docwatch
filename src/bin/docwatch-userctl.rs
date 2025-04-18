use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePoolOptions;
use docwatch::users::UserManager;

#[derive(Parser)]
#[command(name = "userctl", about = "User management CLI for your app")]
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
	List,
	Exists {
		#[arg(short, long)]
		username: String,
	},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();

	let pool = SqlitePoolOptions::new()
		.max_connections(1)
		.connect("sqlite://data/docwatch.db")
		.await?;
	let manager = UserManager::new(pool);

	match cli.command {
		Commands::Create { username, password } => {
			manager.create_user(&username, &password).await?;
			println!("✅ User created: {}", username);
		}
		Commands::Delete { username } => {
			manager.delete_user(&username).await?;
			println!("🗑️  User deleted: {}", username);
		}
		Commands::UpdatePassword { username, password } => {
			manager.update_password(&username, &password).await?;
			println!("🔐 Password updated for: {}", username);
		}
		Commands::List => {
			let users = manager.list_users().await?;
			for user in users {
				println!("{}", user);
			}
		}
		Commands::Exists { username } => {
			if manager.user_exists(&username).await? {
				println!("✅ User exists: {}", username);
			} else {
				println!("❌ User not found: {}", username);
			}
		}
	}

	Ok(())
}
