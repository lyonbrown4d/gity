-- Merge request merge state, diff head, and milestone linkage.
ALTER TABLE project_merge_requests ADD COLUMN milestone_id INTEGER;
ALTER TABLE project_merge_requests ADD COLUMN merged_at TIMESTAMP;
ALTER TABLE project_merge_requests ADD COLUMN merged_by_user_id INTEGER;
ALTER TABLE project_merge_requests ADD COLUMN merge_commit_sha TEXT;
ALTER TABLE project_merge_requests ADD COLUMN head_sha TEXT;
ALTER TABLE project_merge_requests ADD COLUMN closed_at TIMESTAMP;
ALTER TABLE project_merge_requests ADD COLUMN squash INTEGER NOT NULL DEFAULT 0;
ALTER TABLE project_merge_requests ADD COLUMN draft INTEGER NOT NULL DEFAULT 0;

CREATE INDEX ix_project_merge_requests_milestone ON project_merge_requests (milestone_id);
