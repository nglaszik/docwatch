use sqlx::{SqlitePool};
use std::time::Duration;
use diff::lines;

use crate::google_api::{get_many_modified_times, get_google_text};

pub async fn poll_loop(pool: SqlitePool) {
	let mut interval = tokio::time::interval(Duration::from_secs(60));

	loop {
		interval.tick().await;
		if let Err(e) = poll_all_docs(&pool).await {
			eprintln!("Polling error: {:?}", e);
		}
	}
}

pub fn diff_texts(old: &str, new: &str) -> String {
	let mut result = String::new();
	for diff in lines(old, new) {
		use diff::Result::*;
		match diff {
			Left(l) => result.push_str(&format!("- {}\n", l)),
			Right(r) => result.push_str(&format!("+ {}\n", r)),
			Both(b, _) => result.push_str(&format!("  {}\n", b)),
		}
	}
	result
}

async fn poll_all_docs(pool: &SqlitePool) -> Result<(), sqlx::Error> {
	
	let modified_map = get_many_modified_times().await.unwrap_or_default();

	for (doc_id, (name, modified_time, export_link)) in modified_map {
		let db_doc = sqlx::query!(
			"SELECT id, last_updated, latest_content FROM documents WHERE doc_id = ?",
			doc_id
		)
		.fetch_optional(pool)
		.await?;
	
		match db_doc {
			Some(db) => {
				if db.last_updated != modified_time {
					let new_content = match get_google_text(&export_link).await {
						Ok(text) => text,
						Err(e) => {
							eprintln!("⚠️ Failed to fetch document text: {}", e);
							continue;
						}
					};
		
					let diff = diff_texts(&db.latest_content.unwrap_or_default(), &new_content);
		
					sqlx::query!(
						"INSERT INTO document_revisions (document_id, revision_time, content, diff)
						 VALUES (?, ?, ?, ?)",
						db.id,
						modified_time,
						new_content,
						diff
					)
					.execute(pool)
					.await?;
		
					sqlx::query!(
						"UPDATE documents SET name = ?, last_updated = ?, latest_content = ?, export_link = ? WHERE id = ?",
						name,
						modified_time,
						new_content,
						export_link,
						db.id
					)
					.execute(pool)
					.await?;
				}
			}
		
			None => {
				// ✅ New doc — insert it
				sqlx::query!(
					"INSERT INTO documents (user_id, doc_id, name, last_updated, export_link, latest_content)
					 VALUES (?, ?, ?, ?, ?, ?)",
					1, // service/system user
					doc_id,
					name,
					modified_time,
					export_link,
					"" // placeholder for latest_content
				)
				.execute(pool)
				.await?;
		
				println!("📄 Inserted new document into database: {}", name);
			}
		}
	}

	Ok(())
}


