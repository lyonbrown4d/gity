-- Project milestones and richer issue metadata.
CREATE TABLE project_milestones (
    id bigint NOT NULL PRIMARY KEY,
    project_id bigint NOT NULL,
    iid bigint NOT NULL,
    title text NOT NULL,
    description text,
    state text NOT NULL DEFAULT 'active',
    start_date date,
    due_date date,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    UNIQUE (project_id, iid),
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);

CREATE INDEX ix_project_milestones_project_state ON project_milestones USING btree (project_id, state);

ALTER TABLE project_issues ADD COLUMN milestone_id bigint;
ALTER TABLE project_issues ADD COLUMN due_date date;
ALTER TABLE project_issues ADD COLUMN weight bigint;
ALTER TABLE project_issues ADD COLUMN confidential bigint NOT NULL DEFAULT 0;
ALTER TABLE project_issues ADD COLUMN closed_by_user_id bigint;
ALTER TABLE project_issues ADD COLUMN closed_at timestamp without time zone;
ALTER TABLE project_issues ADD COLUMN updated_by_user_id bigint;
ALTER TABLE project_issues ADD COLUMN issue_type text NOT NULL DEFAULT 'issue';

CREATE INDEX ix_project_issues_milestone ON project_issues USING btree (milestone_id);
