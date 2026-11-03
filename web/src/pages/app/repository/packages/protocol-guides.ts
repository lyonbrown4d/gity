import type {
  RepositoryPackageDetailView,
  RepositoryPackageFileContentView,
  RepositoryPackageFileView,
  RepositoryPackageVersionDetailView,
  RepositoryPackageVersionView,
  RepositoryPackageView,
} from "@/pages/types";
import { getApiBaseUrl } from "@/lib/api";
import type { PackageProtocolGuideItem } from "./protocol-guide";

export const buildPackageProtocolGuides = (
  repoId: string,
  selectedPackage: RepositoryPackageView | null,
  selectedVersion: RepositoryPackageVersionDetailView | null,
  selectedFile: RepositoryPackageFileView | null,
): PackageProtocolGuideItem[] => {
  const packageName = encodePathSegment(selectedPackage?.name || "my-package");
  const versionValue = encodePathSegment(selectedVersion?.version.version || "0.1.0");
  const filePath = encodePathTail(selectedFile?.file_path || selectedFile?.file_name || "artifact.tar.gz");
  const apiBase = absoluteApiUrl(`/projects/${repoId}/packages`);
  const genericEndpoint = `${apiBase}/generic/${packageName}/${versionValue}/${filePath}`;
  const nugetEndpoint = `${apiBase}/nuget/index.json`;
  const mavenFilePath = "com/example/demo/0.1.0/demo-0.1.0.jar";
  const pypiPackageName = encodePathSegment(selectedPackage?.name || "my_package");
  const npmPackageName = selectedPackage?.name?.startsWith("@") ? selectedPackage.name : "@gity/my-package";
  const npmEncodedPackage = encodePathTail(npmPackageName);

  return [
    {
      id: "generic",
      title: "Generic",
      description: "Upload arbitrary build artifacts through the generic registry endpoint.",
      endpoint: genericEndpoint,
      scopes: ["read_package", "write_package"],
      publishCommands: [
        `curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./artifact.tar.gz ${genericEndpoint}`,
      ],
      installCommands: [
        `curl --location --header "Authorization: Bearer <token>" --output artifact.tar.gz ${genericEndpoint}`,
      ],
    },
    {
      id: "npm",
      title: "npm",
      description: "Use the npm endpoint for package metadata and publish payloads.",
      endpoint: `${apiBase}/npm/${npmEncodedPackage}`,
      scopes: ["read_package", "write_package"],
      publishCommands: ["npm publish --registry " + apiBase + "/npm/"],
      installCommands: ["npm config set @gity:registry " + apiBase + "/npm/", "npm install " + npmPackageName],
    },
    {
      id: "maven",
      title: "Maven",
      description: "Upload Maven artifacts by repository-relative artifact path.",
      endpoint: `${apiBase}/maven/${mavenFilePath}`,
      scopes: ["read_package", "write_package"],
      publishCommands: [
        `curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./demo-0.1.0.jar ${apiBase}/maven/${mavenFilePath}`,
      ],
      installCommands: [
        `mvn dependency:get -DremoteRepositories=gity::::${apiBase}/maven -Dartifact=com.example:demo:0.1.0`,
      ],
    },
    {
      id: "pypi",
      title: "PyPI",
      description: "Expose a simple Python package index and upload distribution files.",
      endpoint: `${apiBase}/pypi/simple`,
      scopes: ["read_package", "write_package"],
      publishCommands: [
        `curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./dist/my_package-0.1.0.tar.gz ${apiBase}/pypi/${pypiPackageName}/${versionValue}/my_package-0.1.0.tar.gz`,
      ],
      installCommands: [`pip install --index-url ${apiBase}/pypi/simple ${selectedPackage?.name || "my-package"}`],
    },
    {
      id: "nuget",
      title: "NuGet",
      description: "Use the NuGet v3 service index and raw package upload endpoint.",
      endpoint: nugetEndpoint,
      scopes: ["read_package", "write_package"],
      publishCommands: [
        `curl --request PUT --header "Authorization: Bearer <token>" --upload-file ./My.Package.0.1.0.nupkg ${apiBase}/nuget/My.Package/0.1.0/My.Package.0.1.0.nupkg`,
      ],
      installCommands: [
        `dotnet nuget add source ${nugetEndpoint} --name gity --username <username> --password <token> --store-password-in-clear-text`,
        "dotnet add package My.Package --version 0.1.0 --source gity",
      ],
    },
  ];
};

const absoluteApiUrl = (path: string): string => {
  const base = getApiBaseUrl().replace(/\/+$/, "");
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  if (/^https?:\/\//i.test(base)) {
    return `${base}${normalizedPath}`;
  }
  if (typeof window === "undefined") {
    return `${base}${normalizedPath}`;
  }
  return `${window.location.origin}${base}${normalizedPath}`;
};

const encodePathSegment = (value: string): string => encodeURIComponent(value).replace(/%2F/gi, "/");

const encodePathTail = (value: string): string =>
  value
    .split("/")
    .map((part) => encodeURIComponent(part))
    .join("/");
