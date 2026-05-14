CREATE INDEX `ix_project_jobs_queue` ON `project_jobs` (`status`, `run_after`, `kind`);
