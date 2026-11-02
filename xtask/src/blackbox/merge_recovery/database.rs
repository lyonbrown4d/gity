use super::*;
use std::process::Command;

/// Runs only against the already isolated, task-owned Compose project.
/// SQL is frozen test DDL or contains validated numeric application IDs only.
pub(super) fn sql(root: &Path, args: &BlackboxArgs, statement: &str) -> Result<String> {
  let mut command = Command::new("docker");
  match args.mode {
    Mode::Distributed => {
      runtime::configure_compose(&mut command, root, args)?;
      // These database/user names match distributed-infrastructure.compose.yaml.
      command.args([
        "exec",
        "--no-TTY",
        "postgres",
        "psql",
        "-X",
        "-v",
        "ON_ERROR_STOP=1",
        "-U",
        "gity",
        "-d",
        "gity",
        "-A",
        "-t",
        "-f",
        "-",
      ]);
    }
    Mode::Standalone => {
      let mut find = Command::new("docker");
      runtime::configure_compose(&mut find, root, args)?;
      find.args(["ps", "--quiet", "gity"]);
      let id = String::from_utf8(
        process::run_output(&mut find, "locate isolated SQLite container")?.stdout,
      )?;
      let id = id.trim();
      ensure!(
        !id.is_empty() && id.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid task-owned container ID"
      );
      // The cached image supplies a DB client only, never product runtime code.
      // URI mode=rw prevents silently creating a database at an incorrect path.
      const CLIENT: &str = "import sqlite3,sys\nc=sqlite3.connect('file:/var/lib/gity/data/gity.db?mode=rw',uri=True,timeout=30)\ns=sys.stdin.read()\nif s.lstrip().upper().startswith('SELECT'):\n for row in c.execute(s): print(row[0])\nelse:\n c.executescript(s)\n c.commit()\nc.close()\n";
      command.args([
        "run",
        "--rm",
        "--interactive",
        "--pull",
        "never",
        "--network",
        "none",
        "--volumes-from",
        id,
        "python:3.13.2-bookworm",
        "python3",
        "-c",
        CLIENT,
      ]);
    }
  }
  let output = process::run_with_stdin(
    &mut command,
    statement.as_bytes(),
    "isolated merge recovery database client",
  )?;
  Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

pub(super) fn install(root: &Path, args: &BlackboxArgs, project: i64) -> Result<()> {
  let ddl = match args.mode {
    Mode::Standalone => format!(
      "CREATE TRIGGER gity_bb_merge_fault BEFORE INSERT ON project_audit_events WHEN NEW.project_id={project} AND NEW.event_name='project.merge_request.merged' BEGIN SELECT RAISE(ABORT,'blackbox merge completion fault'); END;"
    ),
    Mode::Distributed => format!(
      "CREATE FUNCTION gity_bb_merge_fault_fn() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.project_id={project} AND NEW.event_name='project.merge_request.merged' THEN RAISE EXCEPTION 'blackbox merge completion fault'; END IF; RETURN NEW; END $$; CREATE TRIGGER gity_bb_merge_fault BEFORE INSERT ON project_audit_events FOR EACH ROW EXECUTE FUNCTION gity_bb_merge_fault_fn();"
    ),
  };
  sql(root, args, &ddl)?;
  Ok(())
}

pub(super) fn remove(root: &Path, args: &BlackboxArgs) -> Result<()> {
  sql(
    root,
    args,
    match args.mode {
      Mode::Standalone => "DROP TRIGGER IF EXISTS gity_bb_merge_fault;",
      Mode::Distributed => {
        "DROP TRIGGER IF EXISTS gity_bb_merge_fault ON project_audit_events; DROP FUNCTION IF EXISTS gity_bb_merge_fault_fn();"
      }
    },
  )?;
  Ok(())
}

pub(super) fn completion_counts(
  root: &Path,
  args: &BlackboxArgs,
  project: i64,
  mr: i64,
) -> Result<()> {
  for query in [
    format!(
      "SELECT COUNT(*) FROM project_audit_events WHERE project_id={project} AND target_id='{mr}' AND event_name='project.merge_request.merged'"
    ),
    format!(
      "SELECT COUNT(*) FROM project_todos WHERE project_id={project} AND target_id='{mr}' AND kind='merge_request_merged'"
    ),
    format!(
      "SELECT COUNT(*) FROM merge_operations WHERE project_id={project} AND merge_request_id={mr} AND status='applied' AND events_delivered=true"
    ),
  ] {
    ensure!(
      sql(root, args, &query)? == "1",
      "merge recovery must produce exactly one completed journal, audit and author notification"
    );
  }
  Ok(())
}
