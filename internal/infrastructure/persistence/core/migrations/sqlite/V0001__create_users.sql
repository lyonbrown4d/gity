CREATE TABLE IF NOT EXISTS "users" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "username" TEXT NOT NULL UNIQUE,
    "display_name" TEXT NOT NULL,
    "email" TEXT UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL
);
