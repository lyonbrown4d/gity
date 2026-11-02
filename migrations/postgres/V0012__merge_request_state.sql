-- Merge request merge state, diff head, and milestone linkage.
ALTER TABLE project_merge_requests ADD COLUMN milestone_id bigint;
ALTER TABLE project_merge_requests ADD COLUMN merged_at timestamp without time zone;
ALTER TABLE project_merge_requests ADD COLUMN merged_by_user_id bigint;
ALTER TABLE project_merge_requests ADD COLUMN merge_commit_sha text;
ALTER TABLE project_merge_requests ADD COLUMN head_sha text;
ALTER TABLE project_merge_requests ADD COLUMN closed_at timestamp without time zone;
ALTER TABLE project_merge_requests ADD COLUMN squash bigint NOT NULL DEFAULT 0;
ALTER TABLE project_merge_requests ADD COLUMN draft bigint NOT NULL DEFAULT 0;

CREATE INDEX ix_project_merge_requests_milestone ON project_merge_requests USING btree (milestone_id);
