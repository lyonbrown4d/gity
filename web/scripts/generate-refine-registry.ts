import { readdir, readFile, writeFile } from "node:fs/promises";
import { relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "@babel/parser";

interface Operation {
  operationId?: string;
  parameters?: { name: string; in: string }[];
}
interface Contract {
  paths: Record<string, Record<string, Operation>>;
}
interface GeneratedFunction {
  file: string;
  parameters: string[];
  routeParameters?: string[];
}
const webRoot = fileURLToPath(new URL("../", import.meta.url));
const generatedRoot = resolve(webRoot, "src/lib/generated");
/** Kept explicit in Orval configuration so generated names and dispatch cannot drift. */
export const operationName = (operation: { operationId?: string }): string => {
  if (!operation.operationId) throw new Error("Every generated Refine operation requires operationId");
  const words = operation.operationId.match(/[a-zA-Z0-9]+/g) ?? [];
  return words.map((word, index) => (index ? word[0].toUpperCase() + word.slice(1) : word)).join("");
};
const sourceFiles = async (directory: string): Promise<string[]> => {
  const entries = await readdir(directory, { withFileTypes: true });
  const children = await Promise.all(
    entries
      .filter((entry) => entry.name !== "model")
      .map((entry) => {
        const path = resolve(directory, entry.name);
        return entry.isDirectory() ? sourceFiles(path) : Promise.resolve(entry.name.endsWith(".ts") ? [path] : []);
      }),
  );
  return children.flat();
};
const routeParameters = (node: unknown): string[] | null => {
  if (!node || typeof node !== "object") return null;
  if (Array.isArray(node)) {
    for (const child of node) {
      const result = routeParameters(child);
      if (result) return result;
    }
    return null;
  }
  const value = node as Record<string, unknown>;
  if (value.type === "TemplateLiteral") {
    const parts = value.quasis as { value: { raw: string } }[];
    if (parts[0]?.value.raw.startsWith("/")) {
      return (value.expressions as { type: string; name?: string }[])
        .map((expression) => {
          if (expression.type !== "Identifier" || !expression.name)
            throw new Error("Unsupported generated URL expression");
          return expression.name;
        })
        .filter((name) => name !== "stringifiedParams");
    }
  }
  for (const child of Object.values(value)) {
    const result = routeParameters(child);
    if (result) return result;
  }
  return null;
};
const functionsIn = (file: string, content: string): [string, GeneratedFunction][] => {
  const source = parse(content, { sourceType: "module", plugins: ["typescript"] });
  return source.program.body.flatMap((statement): [string, GeneratedFunction][] => {
    if (statement.type !== "ExportNamedDeclaration" || statement.declaration?.type !== "VariableDeclaration") return [];
    return statement.declaration.declarations.flatMap((declaration): [string, GeneratedFunction][] => {
      const fn = declaration.init;
      if (declaration.id.type !== "Identifier" || fn?.type !== "ArrowFunctionExpression") return [];
      if (!fn.async && !declaration.id.name.endsWith("Url")) return [];
      const parameters = fn.params.map((parameter) => {
        if (parameter.type !== "Identifier") throw new Error(`Unsupported generated parameter in ${file}`);
        return parameter.name;
      });
      return [
        [
          declaration.id.name,
          { file, parameters, routeParameters: fn.async ? undefined : (routeParameters(fn.body) ?? []) },
        ],
      ];
    });
  });
};
const patternFor = (path: string): { pattern: string; names: string[] } => {
  const names: string[] = [];
  const parts = path.split(/(\{[^}]+\})/).map((part, index, all) => {
    if (part.startsWith("{")) {
      const name = part.slice(1, -1);
      names.push(name);
      return name === "path" && index === all.length - 2 && !all[all.length - 1] ? "(.+)" : "([^/]+)";
    }
    return part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  });
  return { pattern: `^${parts.join("")}$`, names };
};
export const generateRefineRegistry = async (): Promise<void> => {
  const [contractSource, files] = await Promise.all([
    readFile(resolve(webRoot, "openapi.json"), "utf8"),
    sourceFiles(generatedRoot),
  ]);
  const contract = JSON.parse(contractSource) as Contract;
  const sources = await Promise.all(files.map(async (file) => functionsIn(file, await readFile(file, "utf8"))));
  const functions = new Map(sources.flat());
  const entries: { path: string; score: number; code: string }[] = [];
  const known = new Set<string>();
  for (const [path, methods] of Object.entries(contract.paths)) {
    for (const [method, operation] of Object.entries(methods)) {
      if (!["get", "post", "put", "patch", "delete", "head", "options"].includes(method)) continue;
      const name = operationName(operation);
      if (known.has(name)) throw new Error(`Duplicate generated operation: ${name}`);
      known.add(name);
      const fn = functions.get(name);
      if (!fn) throw new Error(`Missing Orval request function ${name}; run Orval before generating the bridge`);
      const { pattern, names } = patternFor(path);
      const modulePath = `./${relative(generatedRoot, fn.file).replace(/\\/g, "/").replace(/\.ts$/, "")}`;
      const helper = functions.get(`get${name[0].toUpperCase()}${name.slice(1)}Url`);
      const pathParameters = helper?.routeParameters;
      if (!pathParameters || pathParameters.length !== names.length)
        throw new Error(`Generated URL parameter mismatch for ${name}`);
      const bodyParameters = fn.parameters.filter(
        (parameter) => !pathParameters.includes(parameter) && parameter !== "options" && parameter !== "params",
      );
      if (bodyParameters.length > 1) throw new Error(`Unsupported generated signature for ${name}`);
      const args = fn.parameters.map((parameter, index) => {
        const pathIndex = pathParameters.indexOf(parameter);
        const value =
          pathIndex >= 0
            ? `input.path[${JSON.stringify(names[pathIndex])}]`
            : parameter === "options"
              ? "input.options"
              : parameter === "params"
                ? "input.query"
                : "input.payload";
        return `${value} as unknown as Parameters<typeof ${name}>[${index}]`;
      });
      const queryNames = [
        ...new Set(
          (operation.parameters ?? [])
            .filter((parameter) => parameter.in === "query")
            .map((parameter) => parameter.name),
        ),
      ];
      entries.push({
        path,
        score: path.split("/").filter((part) => part && !part.startsWith("{")).length,
        code: `  { method: ${JSON.stringify(method.toUpperCase())}, path: ${JSON.stringify(path)}, pattern: new RegExp(${JSON.stringify(pattern)}), names: ${JSON.stringify(names)}, queryNames: ${JSON.stringify(queryNames)}, acceptsQuery: ${fn.parameters.includes("params")}, invoke: async (input) => { const { ${name} } = await import(${JSON.stringify(modulePath)}); return ${name}(${args.join(", ")}); } },`,
      });
    }
  }
  entries.sort((left, right) => right.score - left.score || left.path.localeCompare(right.path));
  const target = resolve(generatedRoot, "refine-registry.ts");
  const content = `// Generated from the OpenAPI contract and Orval request functions. Do not edit.\nimport type { GeneratedOperation } from "@/providers/refine/generated-dispatch";\nexport const generatedOperations: readonly GeneratedOperation[] = [\n${entries.map((entry) => entry.code).join("\n")}\n];\n`;
  await writeFile(target, content);
};
