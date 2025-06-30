-- Your SQL goes here
ALTER TABLE "posts" ADD COLUMN "created_at" TIMESTAMP NOT NULL;
ALTER TABLE "posts" ADD COLUMN "author_id" INT4 NOT NULL;

CREATE TABLE "users"(
	"id" SERIAL NOT NULL,
	"username" TEXT NOT NULL,
	"password" TEXT NOT NULL,
	"email" TEXT,
	"created_at" TIMESTAMP NOT NULL
);

