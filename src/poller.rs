use sqlx::{SqlitePool};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use diff::{self, Result as DiffResult};
use regex::Regex;

use crate::google_api::{get_google_docs, get_google_text, get_docx_text};

#[derive(Debug)]
pub enum WordChange<'a> {
	Added(&'a str),
	Removed(&'a str),
	Unchanged(&'a str),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "text")]
pub enum OwnedWordChange {
	Added(String),
	Removed(String),
	Unchanged(String),
}

impl<'a> From<WordChange<'a>> for OwnedWordChange {
	fn from(wc: WordChange<'a>) -> Self {
		match wc {
			WordChange::Added(s) => OwnedWordChange::Added(s.to_string()),
			WordChange::Removed(s) => OwnedWordChange::Removed(s.to_string()),
			WordChange::Unchanged(s) => OwnedWordChange::Unchanged(s.to_string()),
		}
	}
}

pub async fn poll_loop(pool: SqlitePool) {
	let mut interval = tokio::time::interval(Duration::from_secs(5*60));

	loop {
		interval.tick().await;
		if let Err(e) = poll_all_docs(&pool).await {
			eprintln!("Polling error: {:?}", e);
		}
	}
}

pub fn count_words_from_diff<'a>(diff: &[WordChange<'a>]) -> (usize, usize) {
	let mut added = 0;
	let mut removed = 0;

	for change in diff {
		match change {
			WordChange::Added(_) => added += 1,
			WordChange::Removed(_) => removed += 1,
			WordChange::Unchanged(_) => {}
		}
	}

	(added, removed)
}

pub fn diff_words<'a>(old: &'a str, new: &'a str) -> Vec<WordChange<'a>> {
	let token_re = Regex::new(r"\n|[^\S\r\n]+|[^\s]+").unwrap();

	let old_tokens: Vec<&str> = token_re.find_iter(old).map(|m| m.as_str()).collect();
	let new_tokens: Vec<&str> = token_re.find_iter(new).map(|m| m.as_str()).collect();

	let diffs = diff::slice(&old_tokens, &new_tokens);

	diffs
		.into_iter()
		.map(|result| match result {
			DiffResult::Left(w) => WordChange::Removed(w),
			DiffResult::Right(w) => WordChange::Added(w),
			DiffResult::Both(w, _) => WordChange::Unchanged(w),
		})
		.collect()
}

async fn poll_all_docs(pool: &SqlitePool) -> Result<(), sqlx::Error> {
	let modified_map = get_google_docs().await.unwrap_or_default();

	for (doc_id, (name, modified_time, export_link, owner_username, mime_type)) in modified_map {
		let db_doc = sqlx::query!(
			"SELECT id, last_updated, latest_content FROM documents WHERE doc_id = ?",
			doc_id
		)
		.fetch_optional(pool)
		.await?;

		match db_doc {
			Some(db) => {
				if db.last_updated != modified_time {
					let latest_content = db.latest_content.unwrap_or_default();

					// Fetch the document's current content (only when needed)
					let new_content = match mime_type.as_str() {
						"application/vnd.google-apps.document" => match get_google_text(&export_link).await {
							Ok(text) => text,
							Err(e) => {
								eprintln!("⚠️ Failed to fetch Google Doc text: {}", e);
								continue;
							}
						},
						"application/vnd.openxmlformats-officedocument.wordprocessingml.document" => match get_docx_text(&doc_id).await {
							Ok(text) => text,
							Err(e) => {
								eprintln!("⚠️ Failed to fetch DOCX content: {}", e);
								continue;
							}
						},
						other => {
							eprintln!("⚠️ Unsupported MIME type: {}", other);
							continue;
						}
					};

					let diff = diff_words(&latest_content, &new_content);
					let (added_words, deleted_words) = count_words_from_diff(&diff);

					if added_words > 0 || deleted_words > 0 {
						println!("New revision found for: {}", name);

						let owned_diff: Vec<OwnedWordChange> = diff.into_iter().map(Into::into).collect();
						let diff_json = serde_json::to_string(&owned_diff)
							.map_err(|e| sqlx::Error::ColumnDecode {
								index: "diff_json".into(),
								source: Box::new(e),
							})?;
							
						let added_words = added_words as i64;
						let deleted_words = deleted_words as i64;

						sqlx::query!(
							"INSERT INTO document_revisions (
								document_id, revision_time, content, diff, added_words, deleted_words
							) VALUES (?, ?, ?, ?, ?, ?)",
							db.id,
							modified_time,
							new_content,
							diff_json,
							added_words,
							deleted_words
						)
						.execute(pool)
						.await?;

						sqlx::query!(
							"UPDATE documents
							 SET name = ?, last_updated = ?, latest_content = ?, export_link = ?, owner_username = ?
							 WHERE id = ?",
							name,
							modified_time,
							new_content,
							export_link,
							owner_username,
							db.id
						)
						.execute(pool)
						.await?;
					}
				}
			}
			None => {
				// 📄 New document — must always fetch content
				let new_content = match mime_type.as_str() {
					"application/vnd.google-apps.document" => match get_google_text(&export_link).await {
						Ok(text) => text,
						Err(e) => {
							eprintln!("⚠️ Failed to fetch Google Doc text: {}", e);
							continue;
						}
					},
					"application/vnd.openxmlformats-officedocument.wordprocessingml.document" => match get_docx_text(&doc_id).await {
						Ok(text) => text,
						Err(e) => {
							eprintln!("⚠️ Failed to fetch DOCX content: {}", e);
							continue;
						}
					},
					other => {
						eprintln!("⚠️ Unsupported MIME type: {}", other);
						continue;
					}
				};

				// Insert into `documents` with owner_username
				let res = sqlx::query!(
					"INSERT INTO documents (
						doc_id, name, last_updated, export_link, latest_content, owner_username
					) VALUES (?, ?, ?, ?, ?, ?)",
					doc_id,
					name,
					modified_time,
					export_link,
					new_content,
					owner_username
				)
				.execute(pool)
				.await?;

				let new_doc_id = res.last_insert_rowid();

				sqlx::query!(
					"INSERT INTO document_revisions (
						document_id, revision_time, content, diff, added_words, deleted_words
					) VALUES (?, ?, ?, ?, ?, ?)",
					new_doc_id,
					modified_time,
					new_content,
					"", // no diff for initial version
					0,
					0
				)
				.execute(pool)
				.await?;

				println!("📄 Inserted new document and baseline revision: {}", name);
			}
		}
	}

	Ok(())
}



