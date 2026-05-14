CREATE TABLE IF NOT EXISTS "organization_members" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "organization_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
