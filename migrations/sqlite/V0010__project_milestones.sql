-- Project milestones and richer issue metadata.
CREATE TABLE project_milestones (
    id INTEGER NOT NULL PRIMARY KEY,
    project_id INTEGER NOT NULL,
    iid INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    state TEXT NOT NULL DEFAULT 'active',
    start_date TEXT,
    due_date TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE (project_id, iid),
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);

CREATE INDEX ix_project_milestones_project_state ON project_milestones (project_id, state);

ALTER TABLE project_issues ADD COLUMN milestone_id INTEGER;
ALTER TABLE project_issues ADD COLUMN due_date TEXT;
ALTER TABLE project_issues ADD COLUMN weight INTEGER;
ALTER TABLE project_issues ADD COLUMN confidential INTEGER NOT NULL DEFAULT 0;
ALTER TABLE project_issues ADD COLUMN closed_by_user_id INTEGER;
ALTER TABLE project_issues ADD COLUMN closed_at TIMESTAMP;
ALTER TABLE project_issues ADD COLUMN updated_by_user_id INTEGER;
ALTER TABLE project_issues ADD COLUMN issue_type TEXT NOT NULL DEFAULT 'issue';

CREATE INDEX ix_project_issues_milestone ON project_issues (milestone_id);
