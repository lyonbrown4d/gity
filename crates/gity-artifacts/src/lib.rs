//! Domain services for language-specific package registries and generic artifacts.
mod app_error;
mod cargo;
mod deletion;
mod domain;
mod error;
mod go;
mod module;
mod npm;
mod npm_tags;
mod nuget;
mod package_detail;
mod python;
mod record_upload;
mod repository;

pub use cargo::{CargoMetadata, cargo_index_path, cargo_metadata};
pub use domain::{ArtifactCoordinate, PackageFormat};
pub use error::ArtifactValidationError;
pub use go::{go_escape_path, go_unescape_path};
pub use module::{ArtifactModule, ArtifactService};
pub use npm::{npm_manifest, sha256_integrity};
pub use nuget::{NugetDependency, NugetDependencyGroup, NugetMetadata, nuget_metadata};
pub use python::{PythonMetadata, python_metadata};
pub use record_upload::RecordArtifactUpload;
pub use repository::ArtifactRepository;

#[cfg(test)]
#[path = "repository_tests.rs"]
mod repository_tests;
