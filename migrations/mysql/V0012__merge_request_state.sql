-- Merge request merge state, diff head, and milestone linkage.
ALTER TABLE project_merge_requests ADD COLUMN milestone_id bigint NULL;
ALTER TABLE project_merge_requests ADD COLUMN merged_at datetime(6) NULL;
ALTER TABLE project_merge_requests ADD COLUMN merged_by_user_id bigint NULL;
ALTER TABLE project_merge_requests ADD COLUMN merge_commit_sha varchar(64) NULL;
ALTER TABLE project_merge_requests ADD COLUMN head_sha varchar(64) NULL;
ALTER TABLE project_merge_requests ADD COLUMN closed_at datetime(6) NULL;
ALTER TABLE project_merge_requests ADD COLUMN squash bigint NOT NULL DEFAULT 0;
ALTER TABLE project_merge_requests ADD COLUMN draft bigint NOT NULL DEFAULT 0;

CREATE INDEX ix_project_merge_requests_milestone ON project_merge_requests (milestone_id);
