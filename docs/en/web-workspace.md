<div lang="en">

[简体中文](../zh/web-workspace.md) · [English](web-workspace.md)

# Web workspace and themes

Web uses React, React Router, Refine, shadcn, and Tailwind CSS. The workspace shows real pending work, projects, and activity. Repository browsing provides branch search, a file tree, and Monaco. Merge requests expose diffs, inline discussions, approvals, and merge requirements.

## Pages and requests

Project routes start at `/app/projects/{organizationId}/{projectId}`. Code, issues, reviews, and CI have separate routes; filters remain in query parameters. The editor lives at `pipelines/editor`, runs at `pipelines/{pipelineId}`, and job logs at `pipelines/{pipelineId}/jobs/{jobId}`. Independent `jobs/{jobId}` links also work.

Pages query and mutate through Refine. The provider matches a build-time operation registry and invokes actual Orval functions. One Axios transport handles credentials, single-flight refresh, cancellation, locale headers, and HTTP errors. Unknown operations and parameters fail explicitly instead of falling back to handwritten HTTP.

From the repository root:

```sh
cargo run -p gity-server -- openapi --output web/openapi.json
pnpm --dir web gen:api
pnpm --dir web lint
pnpm --dir web test
pnpm --dir web build
```

Generated contracts, clients, and the registry remain untracked. Release builds still expose no Swagger documentation endpoint. ESLint requires arrow functions in handwritten code; generated functions are not edited.

## User themes

The top-bar palette settings offer light, dark, system mode, and Mineral, Ocean, Iris, and Rose presets. Users can customize accent, page and sidebar backgrounds, and corner radius. Changes persist immediately in the current browser and can be reset; they do not provide cross-device account synchronization.

shadcn, code and diff editors, logs, and React Flow share semantic tokens. Custom accents are adjusted to readable tones and buttons receive a matching foreground. Status uses text and icons as well as color. Theme changes update mounted Monaco editors.

## Context and interaction

Packages upload in a modal. Selecting or dropping one file supplies its name, content type and size, and suggests a package name and version from the filename. Users confirm or edit the package identity; package type and storage path are under advanced options. react-dropzone manages file selection, while React Hook Form, Zod and shadcn Field manage the form. Upload failures preserve the file and input with an in-dialog error. Uploading prevents duplicate submissions and dismissal; success refreshes the list and selects the uploaded package.

Wiki creation uses a modal. The page URL defaults from the title, while the server derives authors and editors from the authenticated session. Organization administrators select members by name, username or email instead of entering user IDs. Loading and submission errors appear inside the active dialog.

Work items support keyboard selection; filters, search and selection persist in the URL. Code browsing and the CI editor share a searchable branch popover with arrow-key, Enter and Escape support. Review details separate discussion and file changes, preserving the current comment draft when switching sections. Merging requires source/target confirmation and still obeys server authorization and merge gates. CI property panels keep apply and delete controls at the bottom while fields scroll independently.

## CI editing and execution

React Flow edits tasks and dependency edges in `.gity-ci.yml`; Monaco provides its YAML view. Task properties match the Rust contract: stage, scripts, image, dependencies, variables, runner tags, timeout, and artifact paths. Missing `needs` follows previous stages, while `needs: []` is immediately ready; switching views preserves this distinction.

Validation checks unknown fields, invalid dependencies, cycles, and stage ordering. Saving requires branch permission and the commit SHA originally loaded. A changed branch returns 409 and requires reloading. Editor commits send `skip_pipeline: true`, preserving indexing and events without starting a pipeline. Unsaved edits have navigation confirmation and per-tab draft recovery.

Run details show a read-only task graph from the pinned configuration. Logs load by attempt and byte segment, defaulting to 64 KiB. Users can pause following, search the current segment, change attempts, and download segments. Browsers with the native file-save API can download the full log incrementally; other browsers explicitly retain segment download without assembling the full log in memory.

Verify permissions, failures, and browser behavior separately. Successful builds do not establish acceptance across every browser, database, or distributed deployment.

</div>
