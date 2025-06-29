-- Your SQL goes here
CREATE TABLE "posts"(
	"id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	"title" TEXT NOT NULL,
	"body" TEXT NOT NULL,
	"published" BOOL NOT NULL
);

