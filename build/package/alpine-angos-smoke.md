# Alpine Angos compatibility smoke

Run on 2026-11-01 on linux/amd64 with the pinned official Angos v1.12.4
executable from image index digest
`sha256:1b90f6720c43eb143f519aec4c7bb721f180e74536634b64912085e464c65abd`.

`file` reported an x86-64 static PIE executable. `readelf -l` showed no
`INTERP` program header, so it has no dynamic musl/glibc loader dependency. The
official executable's `--help` command exited successfully in `alpine:3.22`.
The temporary Alpine test image copied only that executable and a filesystem
store config; it did not build or include Angos source.

| Check | Result |
|---|---|
| Angos server startup in `alpine:3.22` | Passed |
| HTTP `GET /v2/` | HTTP 200 with `Docker-Distribution-Api-Version: registry/2.0` |
| Docker push | Passed; manifest digest `sha256:8fd360f67cdb2d528368cbbe47c915a165f04536ff650cb765889bf4e0d44f21` |
| Remove local image tag, then Docker pull | Passed; Docker fetched the manifest and one previously uncached layer |

This confirms the official amd64 binary runs as an Alpine 3.22 OCI registry.
It does not verify the complete Gity image-role matrix or other CPU
architectures.
