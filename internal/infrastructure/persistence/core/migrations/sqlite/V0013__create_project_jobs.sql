CREATE TABLE IF NOT EXISTS "project_jobs" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "project_id" INTEGER NOT NULL,
    "kind" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "payload" TEXT,
    "result" TEXT,
    "attempts" INTEGER NOT NULL,
    "max_attempts" INTEGER NOT NULL,
    "run_after" TIMESTAMP NOT NULL,
    "locked_by" TEXT,
    "locked_until" TIMESTAMP NOT NULL,
    "last_error" TEXT,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "started_at" TIMESTAMP NOT NULL,
    "finished_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);
