CREATE TABLE IF NOT EXISTS "user_access_tokens" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "user_id" BIGINT NOT NULL,
    "name" TEXT NOT NULL,
    "token" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
