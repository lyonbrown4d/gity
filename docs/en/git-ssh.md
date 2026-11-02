<div lang="en">

[简体中文](../zh/git-ssh.md) · [English](git-ssh.md)

# Git SSH Service

SSH uses russh to serve Git transport on a separate TCP port, with username fixed to `git`. It does not depend on system sshd or system accounts. Only `git-upload-pack` and `git-receive-pack` are allowed; shell, SFTP, and port forwarding are unavailable.

## Enable SSH and Preserve Host Identity

Server configuration uses TOML. Merge the following into the actual configuration file. After separate database migration, start with `gity-server --config /path/to/gity.toml serve`. Environment variables use the `GITY_SSH__` prefix, for example `GITY_SSH__ADDRESS=0.0.0.0:2228`.

```toml
[ssh]
enabled = true
address = "0.0.0.0:2222"
host_key = "/var/lib/gity/ssh_host_ed25519"
max_connections = 64
operation_timeout_seconds = 900
```

The code default disables SSH. The provided standalone and distributed Git server Compose configurations enable it and publish host port `${GITY_SSH_PORT:-2222}`. Set firewall and ingress scope for the deployment. The HTTP port does not carry SSH.

An Ed25519 host key is created on first enable; an existing key is reused. On Unix it must be a regular file inaccessible to group and other users, for example mode `0600`. In Compose it resides in standalone's `gity-data` volume or distributed deployment's `git-server-data` volume. Preserve it during recovery. Clients must verify its fingerprint through a trusted channel and pin it in `known_hosts`; accepting an unknown replacement key is not a recovery check.

## Personal SSH Public Keys

Register a public key in the account page's SSH key section or through these self-service endpoints:

| Method and path | Behavior |
| --- | --- |
| `GET /api/v1/users/me/ssh-keys` | List the account's public keys, fingerprints, creation times, and last-use times. |
| `POST /api/v1/users/me/ssh-keys` | Register a key with `title` and `public_key`. |
| `DELETE /api/v1/users/me/ssh-keys/{id}` | Revoke an owned key; missing or other accounts' keys return not found. |

```json
{"title":"work-laptop","public_key":"ssh-ed25519 AAAA..."}
```

Ownership comes from the authenticated account; callers cannot supply another user ID. Administrator privileges do not expose cross-account key management. Private keys always remain on the client and are not accepted by the server.

A personal key represents an account. Every Git command rechecks key existence, account activity, project validity, and current project role and visibility. New channels on an already authenticated SSH connection are checked again. Private projects require access; pushing requires at least developer permission, and protected branches use the account's actual role. Revocation, account disablement, and membership downgrades affect subsequent commands; they do not cancel a transfer that has already been authorized and started.

## Project Deploy Keys

A project administrator registers a project credential through `POST /api/v1/projects/{id}/deploy-keys`:

```json
{"title":"build-host","public_key":"ssh-ed25519 AAAA...","can_push":false}
```

Read-only deploy keys allow clone, fetch, and pull. Enabling `can_push` permits writes to their project. Deploy keys do not inherit their creator's administrator or owner role; protected branches evaluate them at developer level. Every Git command rechecks the project, key, and creator state. Deleting the key or disabling its creator denies subsequent commands. A deploy key does not thereby gain access to other projects.

Both key types are parsed and canonicalized with russh and identified by standard SHA-256 fingerprints. Changing a comment does not create a new identity. Private keys, `authorized_keys` options, and invalid public keys are rejected. Personal fingerprints cannot be registered twice or shared with deploy keys. If concurrent registration produces ambiguity across the two modes, SSH authentication is denied.

## Clients and Protection Rules

Prepare the host key record through a trusted channel, then select a private key and require strict host identity checking:

```sshconfig
Host gity.example
  User git
  Port 2222
  IdentityFile ~/.ssh/id_ed25519
  IdentitiesOnly yes
  StrictHostKeyChecking yes
  UserKnownHostsFile ~/.ssh/gity_known_hosts
```

```sh
git clone ssh://git@gity.example:2222/team/demo.git
cd demo
git push origin main
git pull --ff-only origin main
```

Replace the domain, port, and project path. Protection rules cover existing and new branches, wildcard rules, force pushes, and deletion; the default branch cannot be deleted. Client environment settings cannot override server hook policy. `refs/gity/**` is reserved for server merge receipts: client updates are rejected and HTTP/SSH advertisements hide these references. See [merge recovery](merge-recovery.md).

## Public Revisions and Internal Receipts

Public REST repository reads, archives, and source revisions for new branches or tags may resolve only to commits reachable from public `refs/heads/*` or `refs/tags/*`. Full SHAs, abbreviated SHAs, and revision expressions remain valid for public history. Project access checks still apply.

Direct references to `refs/gity/**` are rejected. Commits retained only by internal receipts and unapplied prepared candidates cannot be read through public APIs or used as branch/tag starting points, even through a known full SHA, abbreviation, or revision expression. Internal receipt reads and complete backups or replication can still access these objects explicitly. Hiding advertisements alone does not replace this reachability check.

## Resource Limits and Verification

Transport is streamed without buffering the complete repository pack. Connection count, a 15-second total public-key authentication wait, and per-operation time are bounded. Disconnect, cancellation, or timeout reaps Git child processes. Logs and `gity_ssh_operations_total` / `gity_ssh_operation_seconds` record outcomes without private keys or public-key authentication signatures.

Related source-test commands:

```sh
cargo test -p gity-platform --lib ssh --locked
cargo test -p gity-git --test ssh --locked
```

These tests have a different scope from image black-box acceptance. See [verification status](verification.md) for actual SSH, authorization-isolation, and recovery evidence across Standalone/distributed × Debian/Alpine.

This reachability rule applies to REST and application revision resolution. Native Git protection checks reference names: a writer who already knows an object SHA may reference it from an ordinary branch they can write. Hidden references therefore do not provide object confidentiality within one repository; receipts must not contain secrets.

</div>
