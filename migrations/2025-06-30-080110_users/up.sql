-- Your SQL goes here
CREATE TABLE "users"
(
	"id" SERIAL PRIMARY KEY,
	"username" TEXT NOT NULL,
	"password" TEXT NOT NULL,
	"email" TEXT,
	"created_at" TIMESTAMP NOT NULL
);

CREATE TABLE "posts"
(
	"id" SERIAL PRIMARY KEY,
	"author_id" INT4 NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"title" TEXT NOT NULL,
	"body" TEXT NOT NULL,
	"published" BOOL NOT NULL,
	FOREIGN KEY ("author_id") REFERENCES "users"("id")
);

