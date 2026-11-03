/** Suggestions only: the user can adjust package identity before publishing. */
export const suggestPackageMetadata = (fileName: string) => {
  const stem = fileName.replace(/\.(?:tar\.(?:gz|bz2|xz|zst)|tgz|zip|gz|whl|nupkg|jar|pom|bin|txt|json)$/i, "");
  const versioned = /^(.*?)[-_]v?(\d+\.\d+\.\d+)$/.exec(stem);
  const type = /\.nupkg$/i.test(fileName)
    ? "nuget"
    : /\.whl$/i.test(fileName)
      ? "pypi"
      : /\.(?:jar|pom)$/i.test(fileName)
        ? "maven"
        : "generic";
  return { name: versioned?.[1] || stem, version: versioned?.[2] || "0.1.0", type };
};
