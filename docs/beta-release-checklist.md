# Beta Release Checklist

This checklist is the release gate for `0.1.x` beta tags.

## Local Verification

Run the release check script before creating a tag:

```powershell
.\scripts\release-check.ps1
```

The script runs:

- `go test ./...`
- `golangci-lint run`
- `pnpm -C web install --frozen-lockfile`
- `pnpm -C web build`
- `goreleaser check`
- `goreleaser release --snapshot --clean --skip=publish,docker`
- Optional manual beta smoke checklist review from [beta-smoke.md](beta-smoke.md)

Local prerequisites for the full script:

- Go from `go.mod`
- pnpm
- golangci-lint v2
- GoReleaser v2
- UPX

When GoReleaser or UPX is not installed locally, run the partial check and rely on GitHub Actions for the package snapshot:

```powershell
.\scripts\release-check.ps1 -SkipGoReleaser
```

If golangci-lint is not installed locally, GitHub Actions still runs lint on pushes and tags:

```powershell
.\scripts\release-check.ps1 -SkipGoLint -SkipGoReleaser
```

## Beta Gate Checklist (Roadmap Items 1-4)

Before any beta tag:

- [ ] Run `.\\scripts\\release-check.ps1 -SkipBetaSmoke` locally and confirm script exit code `0`.
- [ ] Complete all steps in [beta-smoke.md](beta-smoke.md), including:
  - protected branch write policy
  - MR create -> approval -> merge
  - pipeline lifecycle and traces
  - audit event verification
- [ ] Confirm runner runtime assumptions and security notes are documented in [docs/runner-container-runtime.md](docs/runner-container-runtime.md).
- [ ] Confirm API usage examples for:
  - [ ] `project members` lifecycle
  - [ ] `merge-request approval rules`
  - [ ] `CI variables` (masked behavior)

## Smoke Verification

Run the sqlite smoke path from [beta-smoke.md](beta-smoke.md) before tagging.

Required manual checks:

- `gity-standalone` runs migration, server, and worker in one process.
- Split `gity-migration`, `gity-server`, and `gity-worker` can run with the same `.env`.
- A user can create an organization and project.
- HTTP Git fetch and push work through the generated project path.
- A protected branch blocks direct writes when merge requests are required.
- A merge request can be opened, approved, merged, and audited.
- A runner can claim a job, append trace, upload artifact, and complete the job.

## Tagging

Create an annotated prerelease tag:

```powershell
git tag -a v0.1.0-beta.1 -m "v0.1.0-beta.1"
git push origin v0.1.0-beta.1
```

GitHub Actions publishes:

- GitHub Release artifacts
- checksums
- Linux `deb`, `rpm`, and `apk` packages for each binary
- GHCR Docker images for `server`, `migration`, `worker`, `standalone`, and `runner`

The release workflow installs UPX and compresses Linux and Windows binaries before packaging.

## Release Notes Boundary

Call this release a beta. The release notes should state these limitations:

- Runner execution is trusted-host by default.
- Firecracker executor support is experimental and not a complete VM runner yet.
- Package registry support is a baseline, not every ecosystem protocol.
- Search indexing is available, but ranking and incremental invalidation are still early.
- Backup, restore, retention, and HA are not production-complete.
