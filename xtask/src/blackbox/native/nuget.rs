use super::super::{Reporter, process};
use super::{Clients, container};
use anyhow::Result;

pub(super) fn run(context: &Clients, reporter: &mut Reporter) -> Result<()> {
  let root = context.work.join("nuget");
  std::fs::create_dir_all(&root)?;
  let package = format!("Gity.Bb{}", context.suffix);
  let source = format!(
    "{}/projects/{}/packages/nuget/index.json",
    context.api_url, context.project_id
  );
  let mut command = container(
    context,
    "mcr.microsoft.com/dotnet/sdk:10.0.100",
    &root,
    &[
      ("NUGET_PACKAGE", &package),
      ("NUGET_SOURCE", &source),
      ("GITY_TOKEN", &context.token),
    ],
  );
  command.args([
    "-ec",
    r#"
dotnet new classlib -n "$NUGET_PACKAGE" -o /work/publisher --no-restore
dotnet pack /work/publisher -o /work/packages -p:PackageId="$NUGET_PACKAGE" -p:Version=1.0.0
dotnet nuget add source "$NUGET_SOURCE" --name gity --username token --password "$GITY_TOKEN" --store-password-in-clear-text --allow-insecure-connections
dotnet nuget push /work/packages/*.nupkg --source gity --api-key "$GITY_TOKEN"
dotnet new console -n Consumer -o /work/consumer --no-restore
dotnet add /work/consumer package "$NUGET_PACKAGE" --version 1.0.0 --source "$NUGET_SOURCE"
printf 'System.Console.WriteLine(typeof(%s.Class1).FullName);\n' "$NUGET_PACKAGE" > /work/consumer/Program.cs
dotnet build /work/consumer --source "$NUGET_SOURCE"
dotnet run --no-build --project /work/consumer
"#,
  ]);
  process::run_status(&mut command, "native NuGet push and restore")?;
  reporter.pass(
    "native-nuget-push-restore",
    format!("pushed and restored {package} with the .NET NuGet client"),
  );
  Ok(())
}
