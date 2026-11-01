<div lang="en">

[简体中文](../../zh/reference/oci-client.md) · [English](oci-client.md)

# OCI client dependency

The mirror manifest resolver uses the upstream ORAS Rust OCI client, with a
small HTTP transport adapter that enforces outbound host and HTTPS policy.
Gity does not fork or copy client implementation code.

| Field | Value |
| --- | --- |
| Upstream | [oras-project/rust-oci-client](https://github.com/oras-project/rust-oci-client) |
| Crate | `oci-client` |
| Published release | `0.18.0` |
| Pinned source state | `0.19.0` development tree at commit `a913569b1918ea1fda72ed3ba3711b93dea9100f` |
| License | Apache-2.0 |
| Pin | Cargo git dependency with exact `rev`; Cargo.lock records the same commit |

The exact commit is used because its supported `new_with_transport` API lets
Gity validate every registry, Bearer-token realm and redirect destination.
The crates.io `0.18.0` release does not expose that transport API. The source
at the pinned commit had not yet been published as a crates.io release when
this integration was implemented.

</div>
