CREATE TABLE IF NOT EXISTS "project_iid_counters" (
    "project_id" INTEGER NOT NULL,
    "counter_name" TEXT NOT NULL,
    "current_value" INTEGER NOT NULL DEFAULT 0,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    PRIMARY KEY ("project_id", "counter_name"),
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);
