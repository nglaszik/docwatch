use serde_json::Value;
use reqwest;
use std::collections::HashMap;
use anyhow::{anyhow, Result};
use zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{Cursor, Read};

pub async fn get_access_token() -> Result<String, reqwest::Error> {
	let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap();
	let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap();
	let refresh_token = std::fs::read_to_string("google_token.json")
		.ok()
		.and_then(|data| serde_json::from_str::<Value>(&data).ok())
		.and_then(|v| v["refresh_token"].as_str().map(|s| s.to_string()))
		.expect("Missing refresh token");

	let params = [
		("client_id".to_string(), client_id),
		("client_secret".to_string(), client_secret),
		("refresh_token".to_string(), refresh_token),
		("grant_type".to_string(), "refresh_token".to_string()),
	];

	let res = reqwest::Client::new()
		.post("https://oauth2.googleapis.com/token")
		.form(&params)
		.send()
		.await?
		.json::<Value>()
		.await?;

	Ok(res["access_token"].as_str().unwrap().to_string())
}

// unused but keep for later in case useful
// originally used for finding which docs to monitor
// just sharing the doc with an admin user is more streamlined
// no need to mess around with document IDs
// TODO: remove edit capability for service user
pub async fn add_docwatch_property(file_id: &str) -> Result<()> {
	let token = get_access_token().await?;

	let url = format!(
		"https://www.googleapis.com/drive/v3/files/{}?supportsAllDrives=true",
		file_id
	);

	let body = serde_json::json!({
		"properties": {
			"docwatch": "true"
		}
	});

	let res = reqwest::Client::new()
		.patch(&url)
		.bearer_auth(token)
		.header("Content-Type", "application/json")
		.json(&body)
		.send()
		.await?;

	if res.status().is_success() {
		Ok(())
	} else {
		let err = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
		eprintln!("Google API error: {}", err);
		Ok(()) // or return Err(anyhow!(err)) if using anyhow
	}
}

pub async fn get_google_docs() -> Result<HashMap<String, (String, String, String, String, String)>, reqwest::Error> {
	let token = get_access_token().await?;
	let mut results = HashMap::new();
	let mut page_token: Option<String> = None;

	loop {
		let mut url = format!(
			"https://www.googleapis.com/drive/v3/files?q=(mimeType='application/vnd.google-apps.document'+or+mimeType='application/vnd.openxmlformats-officedocument.wordprocessingml.document')+and+trashed=false&fields=files(id,name,modifiedTime,mimeType,owners(displayName,emailAddress),exportLinks),nextPageToken&supportsAllDrives=true&pageSize=1000"
		);

		if let Some(token) = &page_token {
			url.push_str(&format!("&pageToken={}", token));
		}

		let res = reqwest::Client::new()
			.get(&url)
			.bearer_auth(&token)
			.send()
			.await?
			.json::<serde_json::Value>()
			.await?;

		if let Some(files) = res["files"].as_array() {
			for file in files {
				let id = file.get("id").and_then(|v| v.as_str()).unwrap_or("");
				let name = file.get("name").and_then(|v| v.as_str()).unwrap_or("Untitled");
				let modified = file.get("modifiedTime").and_then(|v| v.as_str()).unwrap_or("");
				let mime_type = file.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
				let export_link = file["exportLinks"]["text/plain"].as_str().unwrap_or("");

				let owner_username = file["owners"]
					.as_array()
					.and_then(|owners| owners.get(0))
					.and_then(|owner| {
						owner.get("displayName").and_then(|v| v.as_str())
							.or_else(|| owner.get("emailAddress").and_then(|v| v.as_str()))
					})
					.unwrap_or("unknown");

				if !id.is_empty() && !modified.is_empty() {
					results.insert(
						id.to_string(),
						(
							name.to_string(),
							modified.to_string(),
							export_link.to_string(), // may be blank for DOCX
							owner_username.to_string(),
							mime_type.to_string()
						)
					);
				}
			}
		}

		page_token = res["nextPageToken"].as_str().map(|s| s.to_string());
		if page_token.is_none() {
			break;
		}
	}

	Ok(results)
}

pub async fn get_google_text(export_link: &str) -> Result<String, String> {
	let token = get_access_token().await.map_err(|e| format!("Token error: {}", e))?;

	let client = reqwest::Client::new();
	let text_resp = client
		.get(export_link)
		.bearer_auth(&token)
		.send()
		.await
		.map_err(|e| format!("Failed to download text: {}", e))?;

	if !text_resp.status().is_success() {
		let status = text_resp.status();
		let err_body = text_resp.text().await.unwrap_or_else(|_| "<unreadable>".into());
		return Err(format!(
			"Text export failed: {}\n{}",
			status,
			err_body
		));
	}

	text_resp
		.text()
		.await
		.map_err(|e| format!("Failed to decode text: {}", e))
}

pub async fn get_docx_text(file_id: &str) -> Result<String, String> {
	let token = get_access_token().await.map_err(|e| format!("Token error: {}", e))?;

	let url = format!(
		"https://www.googleapis.com/drive/v3/files/{}?alt=media&supportsAllDrives=true",
		file_id
	);

	let bytes = reqwest::Client::new()
		.get(&url)
		.bearer_auth(&token)
		.send()
		.await
		.map_err(|e| format!("Failed to download DOCX: {}", e))?
		.bytes()
		.await
		.map_err(|e| format!("Failed to read DOCX bytes: {}", e))?;

	let reader = Cursor::new(bytes);
	let mut archive = ZipArchive::new(reader).map_err(|e| format!("Failed to open DOCX zip: {}", e))?;

	let mut document_xml = String::new();
	archive
		.by_name("word/document.xml")
		.map_err(|e| format!("Failed to find word/document.xml: {}", e))?
		.read_to_string(&mut document_xml)
		.map_err(|e| format!("Failed to read document.xml: {}", e))?;

	let mut reader = Reader::from_str(&document_xml);
	reader.trim_text(true);

	let mut text = String::new();

	while let Ok(event) = reader.read_event() {
		match event {
			Event::Text(e) => {
				text.push_str(&e.unescape().unwrap_or_default());
			}
			Event::Eof => break,
			_ => {}
		}
	}

	Ok(text)
}
