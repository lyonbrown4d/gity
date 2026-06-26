# Git Client Stress Tests

The k6 workload covers REST APIs. Git clone/fetch/push needs a real Git client because Smart HTTP uses Git protocol RPCs over `info/refs`, `git-upload-pack`, and `git-receive-pack`.

`scripts/git-client-stress.mjs` seeds a project through the API, creates a project access token with `read_repository` and `write_repository`, then runs concurrent real `git` subprocesses.

## Run Against The Blackbox Stack

```bash
node scripts/git-client-stress.mjs --build --workers 4 --iterations 24
```

The script starts `docker-compose.blackbox.yaml` and keeps the server stack running. Local temporary Git worktrees are cleaned by default.

## Run Against An Existing Server

```bash
node scripts/git-client-stress.mjs --no-compose --base-url http://127.0.0.1:18080 --workers 8 --iterations 80
```

## Workload Modes

```bash
node scripts/git-client-stress.mjs --mode mixed --workers 8 --iterations 100
node scripts/git-client-stress.mjs --mode clone --workers 4 --iterations 20
node scripts/git-client-stress.mjs --mode fetch --workers 16 --iterations 200
node scripts/git-client-stress.mjs --mode push --workers 4 --iterations 40
node scripts/git-client-stress.mjs --mode ls-remote --workers 32 --iterations 500
```

Mixed mode defaults to `clone=20`, `fetch=45`, `push=25`, and `ls-remote=10`. Override with `--clone-percent`, `--fetch-percent`, `--push-percent`, and `--ls-remote-percent`.

## Existing Repository Mode

If you already have a repository URL and token:

```bash
node scripts/git-client-stress.mjs --no-compose --project-url http://127.0.0.1:18080/group/project.git --token <token> --workers 8 --iterations 80
```

The token is used as the password part of HTTP Basic auth. Override the username with `--token-username` if needed.

## Proxy And Credential Helpers

For localhost targets, the runner bypasses Git/HTTP proxies and clears Git credential helpers for subprocesses. This keeps blackbox runs independent from machine-level proxy settings such as `HTTP_PROXY` or Git for Windows credential manager configuration.

The server must return `WWW-Authenticate: Basic` on Git Smart HTTP authentication failures. Git clients use that challenge before retrying URL credentials for private repositories.
## Metrics

The runner prints per-operation count, failures, avg, p50, p95, and max for:

- `setup.init`, `setup.commit`, `setup.push`, `setup.clone`.
- `ls-remote` for ref advertisement/read checks.
- `clone` for fresh client clone flows.
- `fetch` for incremental `git-upload-pack` flows.
- `push` for `git-receive-pack` flows.

`--keep-workdir` preserves local worktrees for debugging. The runner never removes server-side repositories or branches.
