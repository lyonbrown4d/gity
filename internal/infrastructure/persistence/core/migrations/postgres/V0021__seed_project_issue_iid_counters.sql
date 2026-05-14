INSERT INTO "project_iid_counters" ("project_id", "counter_name", "current_value", "created_at", "updated_at")
SELECT "project_id", 'issue', MAX("iid"), CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
FROM "project_issues"
GROUP BY "project_id"
ON CONFLICT ("project_id", "counter_name") DO UPDATE SET
    "current_value" = GREATEST("project_iid_counters"."current_value", EXCLUDED."current_value"),
    "updated_at" = EXCLUDED."updated_at";
