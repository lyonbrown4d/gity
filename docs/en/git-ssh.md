<div lang="en">

[简体中文](../zh/git-ssh.md) · [English](git-ssh.md)

# Git SSH Service

SSH uses [russh](https://docs.rs/russh/0.65.0/russh/server/) to provide Git transport on a separate TCP port. The HTTP port does not carry SSH; SSH does not depend on the system sshd or system user accounts.

## Enable and Configure

Merge the following configuration into the server YAML, then start `gity-server serve`. Environment variables use the `GITY_SSH__` prefix, for example `GITY_SSH__ADDRESS=0.0.0.0:2228`.

```yaml
ssh:
  enabled: true
  address: 0.0.0.0:2222
  host_key: data/ssh_host_ed25519
  max_connections: 64
  operation_timeout_seconds: 900
```

On first enable, an Ed25519 host key is created; an existing key is reused.On Unix, the key file must be a regular file readable and writable only by its owner, for example with `0600` permissions.Persist this file during deployment. Clients should verify the host fingerprint through a trusted channel and add it to `known_hosts`.

In production Compose, the corresponding settings are `GITY_SSH_ENABLED`, `GITY_SSH_BIND`, and `GITY_SSH_PORT`; listening is disabled by default and port mappings bind to localhost only.To allow external access, explicitly set the bind address and network entry point. The host key is in the `app_data` volume.

## Keys and Permissions

Authentication currently uses project deploy keys, and the SSH username is fixed as `git`。A project administrator registers an actual OpenSSH public key through `POST /api/v1/projects/{id}/deploy-keys`:

```json
{"title":"build-host","public_key":"ssh-ed25519 AAAA...","can_push":false}
```

The server parses the key with a third-party library and stores its canonical form and standard SHA-256 fingerprint; changing the comment does not create a new key identity. Data that is valid Base64 but not an actual SSH public key is rejected.

- Read-only deploy keys allow clone, fetch, and pull; setting `can_push` allows writes to their project.
- Keys do not inherit the creator's super-admin status; protected branches use developer-level permission checks.
- The project, key, and creator status are checked again for every Git command; subsequent commands are denied if the key is deleted or its creator disabled.
- Protection rules cover existing and new branches, wildcard rules, force pushes, and deletion; the default branch cannot be deleted.
- SSH permits only `git-upload-pack` and `git-receive-pack`. It does not provide shell, SFTP, or port forwarding, and client environment settings cannot override server hook policies.

Project deploy keys are project credentials. There is currently no personal SSH key management interface. Private keys always remain on the client.

## Client Operations

```sh
git clone ssh://git@gity.example:2222/team/demo.git
cd demo
git push origin main
git pull --ff-only origin main
```

Use SSH client configuration or `GIT_SSH_COMMAND` to select the private key. Replace the example domain, port, and project path with your actual settings.

## Verification

```sh
cargo test -p gity-server --test git_ssh --locked -- --test-threads=1
cargo test -p gity-git --locked
```

Integration tests use temporary directories, random dedicated localhost ports, and strict host-key verification with real system `git` and `ssh` clients. They register keys through the API and cover byte and commit consistency after push/pull between two independent clones, branches and tags, read-only/revoked/cross-project keys, disabled creators, protected branches, shell/SFTP rejection, and child-process cleanup when an individual channel or the service is closed.

Transport is streamed; the full repository pack is not buffered in memory. Connection count, the 15-second total public-key authentication wait, and per-operation time are limited. Disconnect, cancellation, or timeout reaps the Git child process. Logs and the `gity_ssh_operations_total` and `gity_ssh_operation_seconds` metrics record outcomes without recording private keys or public-key authentication signatures.

</div>
