-- Project label definitions replace labels inlined per issue.
CREATE TABLE project_labels (
    id bigint NOT NULL PRIMARY KEY,
    project_id bigint NOT NULL,
    title varchar(255) NOT NULL,
    color varchar(32) NOT NULL DEFAULT '#428BCA',
    description longtext,
    created_at datetime(6) NOT NULL,
    updated_at datetime(6) NOT NULL,
    UNIQUE (project_id, title),
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE project_label_links (
    id bigint NOT NULL PRIMARY KEY,
    project_id bigint NOT NULL,
    label_id bigint NOT NULL,
    target_type varchar(32) NOT NULL,
    target_id bigint NOT NULL,
    created_at datetime(6) NOT NULL,
    UNIQUE (label_id, target_type, target_id),
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (label_id) REFERENCES project_labels (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE INDEX ix_project_label_links_target ON project_label_links (target_type, target_id);

INSERT INTO project_labels (id, project_id, title, color, description, created_at, updated_at)
SELECT MIN(l.id), i.project_id, l.name, COALESCE(MIN(l.color), '#428BCA'), NULL, MIN(l.created_at), MIN(l.updated_at)
FROM project_issue_labels l
JOIN project_issues i ON i.id = l.project_issue_id
GROUP BY i.project_id, l.name;

INSERT INTO project_label_links (id, project_id, label_id, target_type, target_id, created_at)
SELECT l.id, i.project_id, p.id, 'issue', i.id, l.created_at
FROM project_issue_labels l
JOIN project_issues i ON i.id = l.project_issue_id
JOIN project_labels p ON p.project_id = i.project_id AND p.title = l.name;

DROP TABLE project_issue_labels;
