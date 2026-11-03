# Global code and issue search

Authenticated users call `POST /api/v1/search` to search projects they can currently read. Results are grouped into code and issues, with optional organization, project, and resource-type scope. Matching uses case-insensitive Unicode substrings. Code indexing covers default-branch paths and UTF-8 text; issue indexing covers titles and descriptions. Binary files and files larger than `search.max_file_size_bytes` are excluded.

## Permissions and results

Each request reads current account status and project and organization memberships from the database. Public and internal projects follow existing project-read rules; private projects require current membership. Confidential issues are visible to current project Reporters and above, authors, assignees, and active administrators. Authors and assignees must still be able to read the project. Project tokens neither inherit their creator's confidential access nor authenticate user-global search.

Engine queries filter projects, published index namespaces, and confidential visibility before selecting candidates. The service then reloads source issues or Git files at the indexed commit, checks content versions, and rechecks identity and permissions before replying. Membership or token revocation, confidentiality changes, and assignment changes do not wait for reindexing to take effect. Saved todos, activity, and subscriptions also recheck current permissions.

Responses expose no unauthorized resource totals. `snippet` is plain text and must never be rendered as trusted HTML. Code hits include project, path, commit SHA, and line, with one hit per file. Issue hits include the project-scoped IID.

## Requests and pagination

```json
{
  "query": "login failed",
  "types": ["code", "issue"],
  "project_ids": ["123456789012345678"],
  "limit": 20
}
```

The Web quick jump shows the first eight hits per group and prompts users to refine queries when more matches exist. The API supports cursor pagination below.

Omit `project_ids` to search all readable projects. `organization_id` further restricts the scope. `limit` defaults to 20 and accepts 1–50 per group. The trimmed query must be nonempty and at most 512 UTF-8 bytes.

`groups` contains `kind` and `items`. Send `next_cursor` as `cursor` with all other fields unchanged to continue. Cursors expire within 120 seconds and bind the user, request, and permission scope. Changed scope, expired cursors, or changed request fields return 409; restart the search. Revoked credentials return 401. Old cursors cannot retrieve results after membership revocation.

Cursor snapshots are local to the API process, capped at 128 entries, eight per user, and an estimated 64 MiB. Pressure may expire them earlier. Multiple API replicas need sticky routing to continue a cursor; otherwise restart on 409.

A candidate snapshot contains at most 1000 candidates per group. Show `partial: true` and ask users to narrow their search. Engine scan limits and projects without a published index also mark results partial. `indexing_eventually_consistent: true` means new content appears asynchronously; zero hits do not prove that a resource does not exist. Modified source rows are skipped until reindexing. Requests have a 30-second deadline; exhausted concurrency slots, engine failures, or unavailable Git sources return 503.

## Selecting an engine

A single-process standalone deployment defaults to embedded Tantivy:

```toml
[search.global]
enabled = true
backend = "tantivy"
timeout_seconds = 10
```

Persist `search.index_root/global-v1` with the instance. A process lock prevents concurrent use of this directory. Embedded mode supports one application process owning global search for its database; separate local indexes must not publish into the same database. API, Git, and Worker component entry points reject enabled Tantivy configuration.

Distributed components and multiple API replicas use the same OpenSearch cluster and index:

```toml
[search.global]
backend = "opensearch"
endpoint = "https://search.example.internal:9200"
index = "gity-global-v1"
timeout_seconds = 10
```

Inject credentials through `GITY_SEARCH__GLOBAL__USERNAME` and `GITY_SEARCH__GLOBAL__PASSWORD`. The endpoint must not contain URL credentials, query parameters, or fragments. HTTPS uses system-trusted certificates. Redirects are disabled. The remote index stores a projection and never replaces database authorization. It contains source code and confidential text; restrict network access and service-account index permissions.

Distributed Compose includes `public.ecr.aws/opensearchproject/opensearch:3.9.0` with the `opensearch-data` volume and no published host port. Set `OPENSEARCH_INITIAL_ADMIN_PASSWORD`; API, Git, and Worker share this service. The example explicitly sets `allow_insecure_http=true` for internal HTTP with Basic Auth. Use a trusted HTTPS endpoint and a dedicated service account across hosts.

A fixed Gity mapping version is required. Incompatible existing indexes reject reads and writes. Configure a new `index` name and let background tasks rebuild it; do not reuse another system’s index.

Configuration precedence remains defaults, configuration file, environment, then explicit CLI arguments. `enabled=false` disables background global rebuilds and returns 409 from the endpoint.

## Updates and recovery

Issue writes and dirty-project tasks commit in the same transaction. Git write events mark projects dirty. Startup and periodic reconciliation, controlled by `search.refresh_interval_seconds`, recover missed events. Git reads reuse local and remote Git-node adapters.

Tasks coalesce by project with increasing generations and 300-second leases. Each claim writes an isolated namespace. Publication occurs through a conditional database update only after successful indexing. A late worker cannot replace a newer publication. Failed rebuilds retain the previous version and retry after a delay. A changed index volume or remote configuration does not accept another index instance's publication identifiers; startup reconciliation rebuilds the projection.

A rebuild allows at most 100000 issues, 20000 Git tree entries, and 256 MiB of combined issue and code text. `search.max_files_per_project` additionally limits code files. Exceeded budgets or a rebuild exceeding 120 seconds retain the previous version and retry; adjust project content or configurable budgets. Oversized and non-text files are skipped as described above. Each encoded OpenSearch document, including original and search fields, also has a 5 MiB write limit. Exceeding it retains the previous version and records a rebuild failure. Inspect `gity_global_search_jobs_total{result="success|failure"}` and structured logs for rebuild failures. Logs omit query text and credentials.

`crates/gity-search` implements the engines; `crates/gity-platform/src/global_search` coordinates permissions, projection, queries, and cursors; `gity-persistence::search_jobs` persists tasks. Run `cargo test -p gity-search`, `cargo test -p gity-persistence --test search_jobs`, and `cargo test -p gity-platform global_search`. Remote acceptance requires an isolated OpenSearch instance; compilation does not verify remote behavior.
