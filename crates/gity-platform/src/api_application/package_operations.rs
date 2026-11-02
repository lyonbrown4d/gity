//! Package metadata shares application operations with native package protocols.
use super::{AppState, CallArguments, InvocationFuture, OperationRegistration, headers, output};
use gity_api_contracts::ApiContext;
use serde_json::json;

macro_rules! package_operation {
  ($name:ident, $method:literal, $path:literal, |$state:ident, $context:ident, $args:ident| $body:block) => {
    fn $name($state: AppState, $context: ApiContext, $args: CallArguments) -> InvocationFuture {
      Box::pin(async move { $body })
    }
    inventory::submit! {
      OperationRegistration {
        prefix: "/api/v1/projects",
        method: $method,
        path: $path,
        handler: concat!(module_path!(), "::", stringify!($name)),
        invoke: $name,
      }
    }
  };
}
package_operation!(list, "GET", "/{id}/packages", |state, context, args| {
  let project: i64 = args.path()?;
  let query = args.query::<gity_package_registry::control::PackageListQuery>()?;
  let value = gity_package_registry::control::list(
    &crate::package_authority::state(&state),
    &headers(&context),
    project,
    query,
  )
  .await
  .map_err(output::convert)?;
  Ok(gity_core::externalize(value))
});
package_operation!(
  detail,
  "GET",
  "/{id}/packages/{package}",
  |state, context, args| {
    let (project, package) = args.path::<(i64, i64)>()?;
    let value = gity_package_registry::control::detail(
      &crate::package_authority::state(&state),
      &headers(&context),
      project,
      package,
    )
    .await
    .map_err(output::convert)?;
    Ok(gity_core::externalize(value))
  }
);
package_operation!(
  delete_package,
  "DELETE",
  "/{id}/packages/{package}",
  |state, context, args| {
    let (project, package) = args.path::<(i64, i64)>()?;
    gity_package_registry::control::delete_package(
      &crate::package_authority::state(&state),
      &headers(&context),
      project,
      package,
    )
    .await
    .map_err(output::convert)?;
    Ok(json!({"ok":true}))
  }
);
package_operation!(
  delete_version,
  "DELETE",
  "/{id}/packages/{package}/versions/{version}",
  |state, context, args| {
    let (project, package, version) = args.path::<(i64, i64, String)>()?;
    gity_package_registry::control::delete_version(
      &crate::package_authority::state(&state),
      &headers(&context),
      project,
      package,
      &version,
    )
    .await
    .map_err(output::convert)?;
    Ok(json!({"ok":true}))
  }
);
package_operation!(
  delete_file,
  "DELETE",
  "/{id}/packages/files/{file}",
  |state, context, args| {
    let (project, file) = args.path::<(i64, i64)>()?;
    gity_package_registry::control::delete_file(
      &crate::package_authority::state(&state),
      &headers(&context),
      project,
      file,
    )
    .await
    .map_err(output::convert)?;
    Ok(json!({"ok":true}))
  }
);
