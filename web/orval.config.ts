import { defineConfig } from "orval";

export default defineConfig({
  gity: {
    input: "./openapi.json",
    output: {
      target: "./src/lib/generated",
      schemas: "./src/lib/generated/model",
      client: "react-query",
      mode: "tags-split",
      mock: false,
      clean: true,
    },
  },
});
