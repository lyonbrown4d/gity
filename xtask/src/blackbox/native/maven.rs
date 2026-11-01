use super::super::{Reporter, process};
use super::{Clients, container};
use anyhow::Result;
use std::fs;

pub(super) fn run(context: &Clients, reporter: &mut Reporter) -> Result<()> {
  let root = context.work.join("maven");
  let group = "org.gity.blackbox";
  let artifact = format!("fixture-{}", context.suffix);
  let version = "1.0.0";
  let base = format!(
    "{}/projects/{}/packages/maven/",
    context.api_url, context.project_id
  );
  fs::create_dir_all(root.join("classes"))?;
  fs::write(
    root.join("classes/marker.txt"),
    "maven registry roundtrip\n",
  )?;
  fs::write(
    root.join("settings.xml"),
    "<settings><mirrors><mirror><id>maven-default-http-blocker</id><mirrorOf>external:http:*,!gity</mirrorOf><url>http://0.0.0.0/</url><blocked>true</blocked></mirror></mirrors><servers><server><id>gity</id><username>token</username><password>${env.GITY_TOKEN}</password></server></servers></settings>\n",
  )?;
  let mut command = container(
    context,
    "maven:3.9.9-eclipse-temurin-21",
    &root,
    &[("GITY_TOKEN", &context.token), ("MAVEN_BASE", &base)],
  );
  command.args([
    "-ec",
    &format!(
      r#"
jar cf /work/fixture.jar -C /work/classes .
mvn -q -s /work/settings.xml org.apache.maven.plugins:maven-deploy-plugin:3.1.3:deploy-file -DrepositoryId=gity -Durl="$MAVEN_BASE" -Dfile=/work/fixture.jar -DgroupId={group} -DartifactId={artifact} -Dversion={version} -Dpackaging=jar -DgeneratePom=true
rm -rf /root/.m2/repository/org/gity/blackbox
mvn -q -s /work/settings.xml org.apache.maven.plugins:maven-dependency-plugin:3.7.1:get -Dartifact={group}:{artifact}:{version} -DremoteRepositories=gity::default::"$MAVEN_BASE"
cp /root/.m2/repository/org/gity/blackbox/{artifact}/{version}/{artifact}-{version}.jar /work/downloaded.jar
"#
    ),
  ]);
  process::run_status(&mut command, "native Maven deploy and dependency resolve")?;
  anyhow::ensure!(
    fs::read(root.join("fixture.jar"))? == fs::read(root.join("downloaded.jar"))?,
    "native Maven downloaded JAR differs from the published artifact"
  );
  reporter.pass(
    "native-maven-deploy-resolve",
    format!("deployed and resolved {group}:{artifact}:{version} with Maven"),
  );
  Ok(())
}
