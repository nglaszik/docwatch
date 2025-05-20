use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
#[serde(tag = "type", content = "text")]
enum OwnedWordChange {
	Added(String),
	Removed(String),
	Unchanged(String),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let db = SqlitePool::connect("sqlite:/opt/docwatch/data/docwatch.db").await?;

	let revisions = sqlx::query!(
		r#"
		SELECT id, diff
		FROM document_revisions
		"#
	)
	.fetch_all(&db)
	.await?;

	for rev in revisions {
		let diff_str = rev.diff.unwrap_or_default();

		let parsed = serde_json::from_str::<Vec<OwnedWordChange>>(&diff_str);

		if let Ok(changes) = parsed {
			let added = changes
				.iter()
				.filter(|c| matches!(c, OwnedWordChange::Added(w) if !w.trim().is_empty()))
				.count() as i64;

			let removed = changes
				.iter()
				.filter(|c| matches!(c, OwnedWordChange::Removed(w) if !w.trim().is_empty()))
				.count() as i64;

			sqlx::query!(
				r#"
				UPDATE document_revisions
				SET added_words = ?, deleted_words = ?
				WHERE id = ?
				"#,
				added,
				removed,
				rev.id
			)
			.execute(&db)
			.await?;
			
			eprintln!("Successful diffing {}", rev.id);
			
		} else {
			eprintln!("Skipping invalid diff for revision id {}", rev.id);
		}
	}

	println!("Update complete.");
	Ok(())
}