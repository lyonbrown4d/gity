INSERT INTO "project_iid_counters" ("project_id", "counter_name", "current_value", "created_at", "updated_at")
SELECT "project_id", 'merge_request', MAX("iid"), CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
FROM "project_merge_requests"
WHERE 1 = 1
GROUP BY "project_id"
ON CONFLICT ("project_id", "counter_name") DO UPDATE SET
    "current_value" = MAX("project_iid_counters"."current_value", excluded."current_value"),
    "updated_at" = excluded."updated_at";
