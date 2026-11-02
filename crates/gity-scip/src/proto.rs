//! Generated SCIP protobuf types.
//!
//! `build.rs` compiles the vendored `proto/scip.proto` at build time, so the
//! decoder tracks upstream field numbers instead of re-implementing the wire
//! format by hand.
// Generated code covers the whole schema, not only the fields this layer reads,
// and its naming follows the protobuf names verbatim.
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/scip.rs"));
