-- This file should undo anything in `up.sql`
ALTER TABLE "posts" DROP COLUMN "created_at";
ALTER TABLE "posts" DROP COLUMN "author_id";

DROP TABLE IF EXISTS "users";
