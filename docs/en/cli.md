<div lang="en">

[简体中文](../zh/cli.md) · [English](cli.md)

# The gity command line tool

`gity` is the command line client for a Gity server. Its command layout follows GitLab's `glab` and GitHub's `gh`: one YAML file holds the current server and a token per server, and `--host`, `--token`, `--json`, and `--config` are global options accepted before or after any subcommand. The binary is `gity`; the Cargo package is `gity-cli`.

The client only calls the server REST API under the fixed `/api/v1` prefix and keeps no second interface definition of its own. Output defaults to aligned tables or `key: value` lines; `--json` prints the raw response as indented JSON. A table with no rows prints `(no results)`, which is not an error. Colour is applied only when standard output is a terminal and `NO_COLOR` is unset.

## Installation

Install from the repository root:

```bash
cargo install --path apps/gity-cli
gity --version
```

The binary lands in Cargo's bin directory, `~/.cargo/bin/gity` by default. `gity repo clone` runs the local `git`, so Git must be on `PATH`.

`gity completion` writes a shell completion script; the argument is `bash`, `elvish`, `fish`, `powershell`, or `zsh`:

```bash
gity completion bash > /etc/bash_completion.d/gity
gity completion zsh > "${fpath[1]}/_gity"
gity completion fish > ~/.config/fish/completions/gity.fish
```

## Authentication

Both login paths end with a token stored in the configuration file. The first uses an existing personal access token (PAT):

```bash
gity auth login --host http://127.0.0.1:8080 --token gity_xxx
```

The second exchanges a username and password for a fresh PAT:

```bash
gity auth login --host http://127.0.0.1:8080 --username admin --password 'change-me'
```

`--host` may be omitted, in which case the global `--host` or `GITY_HOST` is used; `--token` likewise falls back to `GITY_TOKEN`. The password path stores no password: the client posts the credentials to `/api/v1/auth/login`, takes the session token from the response, uses that session to create a PAT named by `--name` (default `gity-cli`) at `/api/v1/users/{user_id}/tokens`, and keeps only the PAT. `--username` without `--password` fails with `--password is required together with --username`; neither of them and no `--token` fails with `pass --token, or --username and --password`.

Before writing anything, the client verifies the token against `/api/v1/users/me` and then stores the server and token as the current configuration:

```bash
gity auth login --host http://127.0.0.1:8080 --token gity_xxx
# signed in as admin on http://127.0.0.1:8080
# credentials stored in ~/.config/gity-cli/config.yaml
```

`gity auth status` calls `/users/me` and prints the current server, user, and administrator flag:

```bash
gity auth status
# host:  http://127.0.0.1:8080
# user:  admin
# admin: false

gity auth status --json
# {"host": "http://127.0.0.1:8080", "user": { ... }}
```

`gity auth logout` deletes the current server's entry from `hosts` and promotes the first remaining host to current. It clears local credentials only and does not revoke the token on the server; use `gity token revoke` to invalidate a token. With nothing stored it prints `no stored credentials` and still succeeds.

## Configuration

The configuration is a YAML file at `gity-cli/config.yaml` inside the system configuration directory: `~/.config/gity-cli/config.yaml` on Linux and `~/Library/Application Support/gity-cli/config.yaml` on macOS. Use `--config <path>` or `GITY_CONFIG` to point elsewhere. Saving creates the parent directory and sets the file mode to `0600` on Unix.

```yaml
host: http://127.0.0.1:8080
hosts:
  http://127.0.0.1:8080:
    token: gity_xxxxxxxxxxxxxxxx
    user: admin
  http://git.example.com:
    token: gity_yyyyyyyyyyyyyyyy
git_protocol: https
editor: null
```

`host` is the current server. Reads and writes strip a trailing `/`, and a `hosts` key must match the stripped address exactly, otherwise the token for that server is not found. `hosts.<host>.token` is the PAT and `user` is the name recorded at login. `git_protocol` defaults to `https` and accepts only `https` or `ssh`. `editor` defaults to `null`. `gity repo clone` uses the `clone_http_url` advertised by the server; `git_protocol` is a validated, stored setting and does not change the protocol used for cloning.

The `gity config` family reads and writes this file:

| Command | Behaviour |
| --- | --- |
| `gity config get <key>` | Prints `host`, `git_protocol`, or `editor`; other keys fail |
| `gity config set <key> <value>` | Writes one key, saves the file, then prints `key = value` |
| `gity config list` | Prints the file path, current host, `git_protocol`, `editor`, and configured hosts |
| `gity config host add <host> --token <token>` | Stores a token for that server and makes it current |
| `gity config host remove <host>` | Removes that server entry; fails when it is not configured |
| `gity config host list` | Lists configured servers as a `HOST`, `USER`, `AUTH`, `CURRENT` table |

`host add` also accepts `--token` from `GITY_TOKEN`. In `host list` the current server is marked with `*`, and `AUTH` shows `token` when a token is stored.

Environment variables and precedence:

| Variable | Equivalent option |
| --- | --- |
| `GITY_HOST` | `--host` |
| `GITY_TOKEN` | `--token` (also read by `auth login` and `config host add --token`) |
| `GITY_CONFIG` | `--config` |

Precedence, highest first: command line option, environment variable, configuration file. Environment variables act as option defaults, so a value given on the command line wins. Without `--host`/`GITY_HOST` the `host` field is used, and when neither exists the client fails with `no server configured: pass --host or run gity auth login --host <url>`. Without `--token`/`GITY_TOKEN` the token comes only from the current host's `hosts` entry, and otherwise the client fails with `not signed in to <host>: pass --token or run gity auth login`.

## Command reference

### Global options

| Option | Effect |
| --- | --- |
| `--host <url>` | Server address, equivalent to `GITY_HOST` |
| `--token <token>` | Personal access token, equivalent to `GITY_TOKEN`, overrides the file |
| `--json` | Prints the raw response instead of a table; affects only subcommands that print API data, while `gity api` always prints JSON |
| `--config <path>` | Configuration file path, equivalent to `GITY_CONFIG` |

### auth

| Command | Effect |
| --- | --- |
| `gity auth login` | Stores credentials; options are `--host`, `--token`, `--username`, `--password`, `--name` |
| `gity auth status` | Shows the current server, user, and administrator flag |
| `gity auth logout` | Removes the locally stored credentials |

### config

Options and keys are described under [Configuration](#configuration).

### token

| Command | Effect |
| --- | --- |
| `gity token list` | Lists the current user's tokens as `ID`, `NAME`, `CREATED`; the API returns neither the secret nor its hash |
| `gity token create --name <name>` | Creates a token, prints the secret once, and shows `export GITY_TOKEN=...` |
| `gity token revoke <id>` | Revokes a token by id |

### repo

| Command | Effect |
| --- | --- |
| `gity repo clone <project> [directory]` | Runs `git clone` against the advertised `clone_http_url`; `--with-token` embeds the token in the URL |

`<project>` is a numeric id, a `group/project` path, or a project name; the client lists visible projects and matches `full_path`, `path`, or `name`. A missing `directory` uses the project `path_key`, or `project` when that field is empty. `--with-token` builds `http://oauth2:<token>@host/path` from `--token` or `GITY_TOKEN`, and the command fails when neither is set; long-lived checkouts should omit it so credentials can be cleaned up later.

### project

| Command | Effect |
| --- | --- |
| `gity project list` | Lists projects visible to the current user as `ID`, `PATH`, `VISIBILITY`, `NAME` |
| `gity project view <project>` | Prints id, path, name, visibility, default branch, and clone URL line by line |

### issue

| Command | Effect |
| --- | --- |
| `gity issue list <project> [--state <state>]` | Lists issues; `state` is filtered server-side on `status` (`all` disables it) and the page window is pushed to the server; defaults to page 1 with 20 items, response shape `{items,total,page,page_size}` |
| `gity issue view <project> <iid>` | Prints title, state, labels, assignees, milestone, description, and every comment |
| `gity issue create <project> --title <title> [--description <text>]` | Creates an issue |
| `gity issue close <project> <iid>` | Sets the state to `closed` |
| `gity issue comment <project> <iid> --body <text>` | Adds a comment |
| `gity issue assign <project> <iid> <user-id>...` | Replaces the assignee set with the given user ids; at least one is required |
| `gity issue unassign <project> <iid> <user-id>...` | Removes the given users from the current assignees |
| `gity issue labels <project> <iid>` | Prints the labels of one issue |
| `gity issue label <project> <iid> --add <name> --remove <name>` | Adds and removes labels by name; at least one of `--add` or `--remove` is required |
| `gity issue milestone <project> <iid> --set <id>` or `--clear` | Sets or clears the milestone; pass exactly one of the two |
| `gity issue milestones <project>` | Lists project milestones as `IID`, `TITLE`, `STATE`, `DUE` |

### mr

| Command | Effect |
| --- | --- |
| `gity mr list <project> [--state <state>]` | Lists merge requests; `state` is filtered server-side (`all` disables it) and the page window is pushed to the server; defaults to page 1 with 20 items, response shape `{items,total,page,page_size}` |
| `gity mr view <project> <iid>` | Prints title, state, source and target branches, description, and the checks returned by `/checks` |
| `gity mr create <project> --title <title> --source <branch> --target <branch> [--description <text>]` | Opens a merge request |
| `gity mr merge <project> <iid> [--squash]` | Merges the merge request; `--squash` collapses the source commits into one |
| `gity mr diff <project> <iid>` | Prints the diff; text responses are printed verbatim, anything else as JSON |

### pipeline

| Command | Effect |
| --- | --- |
| `gity pipeline list <project>` | Lists pipelines as `ID`, `STATUS`, `REF`, `SHA` |
| `gity pipeline view <project> <pipeline>` | Prints id, status, state, ref, sha, and created_at |
| `gity pipeline jobs <project> <pipeline>` | Lists jobs as `ID`, `NAME`, `STATUS`, `STAGE` |
| `gity pipeline run <project> [--ref <ref>] [--sha <sha>]` | Starts a pipeline with `source: cli`; without a ref the server uses the project default branch, and `--sha` names the commit directly |
| `gity pipeline retry <project> <pipeline>` | Re-runs a finished pipeline with new job attempts |
| `gity pipeline cancel <project> <pipeline>` | Cancels a running pipeline |

### completion

| Command | Effect |
| --- | --- |
| `gity completion <shell>` | Prints a completion script for `bash`, `elvish`, `fish`, `powershell`, or `zsh` |

### api

| Command | Effect |
| --- | --- |
| `gity api <path> [-X <method>] [-f key=value]...` | Calls an `/api/v1` endpoint directly and prints the `data` field of the response |

`path` may start with `/` or be given as `projects`. `-X` accepts `get`, `post`, `patch`, `put`, `delete` and defaults to `get`; `put` sends a PUT request. `-f` is repeatable, and a value becomes a JSON value when it parses as one (`-f organization_id=1` sends a number) and a string otherwise; without `-f` an empty object is sent. The response is always printed as indented JSON, including with `--json`.

## Common workflows

The examples below target a local server at `http://127.0.0.1:8080`.

Sign in and confirm the identity:

```bash
gity auth login --host http://127.0.0.1:8080 --username admin --password 'change-me'
gity auth status
gity config host list
```

Look up the organization, then create a project. `gity` has no dedicated create subcommand, so `gity api` calls the same REST endpoint:

```bash
gity api /orgs
gity api /projects -X post -f name=demo -f path=demo -f organization_id=1 -f visibility=private
gity project list
gity project view demo
```

`POST /projects` requires `name` and `organization_id`, and accepts `path` (the path key, defaulting to `name`), `default_branch` (default `main`), `visibility` (`private`, `internal`, or `public`, default `private`), and `description`.

Clone the repository and start a branch:

```bash
gity repo clone demo
cd demo
git switch -c feature/login
```

File and follow an issue:

```bash
gity issue create demo --title "Login page returns 500" --description "Reproduction steps attached"
gity issue list demo
gity issue view demo 1
gity issue comment demo 1 --body "Traced to session parsing"
gity issue label demo 1 --add bug --add P1
gity issue close demo 1
```

Push the branch, open a merge request, and merge it:

```bash
git push -u origin feature/login
gity mr create demo --title "Fix login page 500" --source feature/login --target main
gity mr view demo 1
gity mr diff demo 1
gity mr merge demo 1 --squash
```

Running a pipeline requires `.gity-ci.yml` on the project default branch:

```bash
gity pipeline run demo --ref main
gity pipeline list demo
gity pipeline jobs demo 1
gity pipeline retry demo 1
```

Read raw JSON from a script:

```bash
gity issue list demo --json | jq '.[].title'
gity api /users/me | jq .username
```

## Exit and errors

A successful command exits with `0`. Any failure is returned from `main`, so the runtime prints `Error: <message>` to standard error and exits with `1`. Server-side failures read `<message> (HTTP <status>)`, for example `insufficient project permissions (HTTP 403)`; a response whose `code` is not `0`, or a non-success status, counts as a failure. Local failures such as an unresolvable project reference print the message alone, for example `project 'demo' not found`.

`gity repo clone` checks the exit status of `git clone` directly and fails with `git clone failed with <status>`; a missing Git binary fails with `run git clone; make sure git is installed`.

Output to a pipe has no colour, so `gity ... | grep` and `jq` are safe. Use `--json`, or `gity api` directly, when a stable machine-readable shape is needed, and never parse the tables. An empty table prints `(no results)` and still exits with `0`.

</div>
