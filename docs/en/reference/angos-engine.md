<div lang="en">

[简体中文](../../zh/reference/angos-engine.md) · [English](angos-engine.md)

# Angos engine asset

Gity embeds the upstream precompiled engine as a private child process. It does
not copy or modify Angos application source.

| Field | Value |
| --- | --- |
| Upstream | [project-angos/angos](https://github.com/project-angos/angos) |
| Release | `v1.12.4` |
| Source commit | `c4cad324360967746fabf3ebf9528601bede7ae6` |
| License | Apache-2.0 |
| Image source | `ghcr.io/project-angos/angos:1.12.4` |
| OCI index SHA-256 | `1b90f6720c43eb143f519aec4c7bb721f180e74536634b64912085e464c65abd` |
| Linux amd64 manifest SHA-256 | `2d9f69b2f6ed652cae70406c2387235406c00c652f08feb7e37f9d8413b41584` |
| Linux arm64 manifest SHA-256 | `7192ea5a7ce9555a2f7c1806a3bd0c0b8921371b73626c9e7d075b26acc54dd3` |
| Executable in upstream image | `/angos` |
| Image revision label | `c4cad324360967746fabf3ebf9528601bede7ae6` |

`build/package/Dockerfile.variants` pins the image by index digest and copies
only `/angos`. The executable is a static PIE with no ELF interpreter. It ran
`--help` successfully in Alpine 3.22, so the Alpine image reuses the official
binary instead of rebuilding upstream source. OCI and full standalone targets
may include it; other variants do not. The upstream release provides Cosign
signatures; verify them when updating this pin.

</div>
