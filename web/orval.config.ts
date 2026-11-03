import { defineConfig } from "orval";
import { generateRefineRegistry, operationName } from "./scripts/generate-refine-registry";

export default defineConfig({
  gity: {
    input: "./openapi.json",
    hooks: { afterAllFilesWrite: generateRefineRegistry },
    output: {
      target: "./src/lib/generated",
      schemas: "./src/lib/generated/model",
      client: "react-query",
      mode: "tags-split",
      mock: false,
      clean: true,
      override: {
        operationName,
        fetch: {
          includeHttpResponseReturnType: false,
        },
        mutator: {
          path: "./src/lib/orval-fetch.ts",
          name: "customFetch",
        },
      },
    },
  },
});
