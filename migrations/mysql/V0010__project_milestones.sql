-- Project milestones and richer issue metadata.
CREATE TABLE project_milestones (
    id bigint NOT NULL PRIMARY KEY,
    project_id bigint NOT NULL,
    iid bigint NOT NULL,
    title varchar(255) NOT NULL,
    description longtext,
    state varchar(32) NOT NULL DEFAULT 'active',
    start_date date NULL,
    due_date date NULL,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    UNIQUE (project_id, iid),
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE INDEX ix_project_milestones_project_state ON project_milestones (project_id, state);

ALTER TABLE project_issues ADD COLUMN milestone_id bigint NULL;
ALTER TABLE project_issues ADD COLUMN due_date date NULL;
ALTER TABLE project_issues ADD COLUMN weight bigint NULL;
ALTER TABLE project_issues ADD COLUMN confidential bigint NOT NULL DEFAULT 0;
ALTER TABLE project_issues ADD COLUMN closed_by_user_id bigint NULL;
ALTER TABLE project_issues ADD COLUMN closed_at datetime(6) NULL;
ALTER TABLE project_issues ADD COLUMN updated_by_user_id bigint NULL;
ALTER TABLE project_issues ADD COLUMN issue_type varchar(32) NOT NULL DEFAULT 'issue';

CREATE INDEX ix_project_issues_milestone ON project_issues (milestone_id);
