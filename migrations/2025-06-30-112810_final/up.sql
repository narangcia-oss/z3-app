-- PostgreSQL-adapted schema
CREATE TABLE users
(
	id SERIAL PRIMARY KEY,
	username TEXT NOT NULL,
	created_at TIMESTAMP NOT NULL
);

CREATE TABLE posts
(
	id SERIAL PRIMARY KEY,
	author_id INTEGER REFERENCES users(id),
	created_at TIMESTAMP NOT NULL,
	title TEXT NOT NULL,
	body TEXT NOT NULL,
	published BOOLEAN NOT NULL
);

CREATE TABLE verification_tokens
(
	identifier TEXT NOT NULL,
	token TEXT NOT NULL,
	expires TIMESTAMP NOT NULL,
	PRIMARY KEY(identifier, token)
);

CREATE TABLE accounts
(
	id SERIAL PRIMARY KEY,
	user_id INTEGER NOT NULL REFERENCES users(id),
	type_ TEXT NOT NULL,
	email TEXT,
	password TEXT,
	provider TEXT,
	provider_account_id TEXT,
	refresh_token TEXT,
	access_token TEXT,
	expires_at INTEGER,
	token_type TEXT,
	scope TEXT,
	id_token TEXT,
	session_state TEXT,
	refresh_token_expires_in INTEGER
);

CREATE TABLE sessions
(
	id TEXT PRIMARY KEY,
	session_token TEXT NOT NULL,
	user_id INTEGER NOT NULL REFERENCES users(id),
	expires TIMESTAMP NOT NULL
);

