-- Create users table.
create table if not exists users
(
	id integer primary key not null,
	username text not null unique,
	password text not null
);

-- Create sessions table.
CREATE TABLE sessions (
	user_id INTEGER NOT NULL,
	token TEXT PRIMARY KEY,
	created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- monitored documents table
CREATE TABLE IF NOT EXISTS documents (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	user_id INTEGER NOT NULL,
	doc_id TEXT NOT NULL UNIQUE,
	last_updated TEXT NOT NULL,
	name TEXT NOT NULL DEFAULT 'Untitled',
	latest_content TEXT,
	export_link TEXT,
	created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
	FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE document_revisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id INTEGER NOT NULL,
  revision_time TEXT NOT NULL,
  content TEXT NOT NULL,
  diff TEXT,
  added_words INTEGER DEFAULT 0,
  deleted_words INTEGER DEFAULT 0,
  FOREIGN KEY(document_id) REFERENCES documents(id)
);
