CREATE TABLE IF NOT EXISTS "project_ci_variables" (
    "id" BIGINT NOT NULL PRIMARY KEY,
    "project_id" BIGINT NOT NULL,
    "key" TEXT NOT NULL,
    "value" TEXT NOT NULL,
    "masked" INTEGER NOT NULL,
    "protected" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("project_id") REFERENCES "projects" ("id") ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_ci_variables_project_key" ON "project_ci_variables" ("project_id", "key");
CREATE INDEX IF NOT EXISTS "ix_project_ci_variables_flags" ON "project_ci_variables" ("masked", "protected");
