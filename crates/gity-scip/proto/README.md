# Vendored SCIP schema

`scip.proto` is copied verbatim from the SCIP project, the wire format the
precision targets emit.

- Upstream: <https://github.com/sourcegraph/scip/blob/main/scip.proto>
- Fetched: 2026-11-02 from `main`
- SHA-256: `b38021b65ef90cbbf6af9c829ff75192859ad9b5da05439ef154bea4ceb2bf03`
- License: Apache-2.0 (SCIP project)

Do not edit the copy. To upgrade, replace the file, update the checksum above,
and rerun `cargo test -p gity-scip`; `build.rs` regenerates the prost types, so
new fields are available without hand-written wire parsing.
