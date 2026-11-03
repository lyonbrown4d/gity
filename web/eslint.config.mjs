import babelParser from "@babel/eslint-parser";

const syntax = (isTSX) => ({
  parser: babelParser,
  parserOptions: {
    requireConfigFile: false,
    babelOptions: { parserOpts: { plugins: isTSX ? ["jsx"] : [] }, plugins: ["@babel/plugin-syntax-typescript"] },
  },
});

export default [
  { ignores: ["node_modules/**", "src/lib/generated/**", "src/components/ui/**", "playwright-report/**", "test-results/**"] },
  { files: ["**/*.{ts,tsx}"], languageOptions: syntax(false) },
  { files: ["**/*.tsx"], languageOptions: syntax(true) },
  {
    files: ["**/*.{js,mjs,ts,tsx}"],
    rules: {
      "no-restricted-syntax": ["error",
        { selector: "FunctionDeclaration", message: "Declare handwritten frontend functions as const arrow functions." },
        { selector: "FunctionExpression", message: "Use an arrow function expression." },
      ],
      "prefer-arrow-callback": "error",
    },
  },
  {
    files: ["src/pages/**/*.{ts,tsx}", "src/components/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": ["error", { patterns: [{
        group: ["@/lib/generated/*/*"], allowTypeImports: true,
        message: "Use Refine hooks through the generated-operation provider; generated DTO type imports are allowed.",
      }], paths: [{ name: "axios", message: "Use the Refine provider and shared Orval transport." }] }],
    },
  },
];
