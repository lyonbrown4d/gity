# Repository initialization options

Project creation can select a license and `.gitignore`, and enable “Initialize with a README.” These options are independent. Selected files form one initial commit on the chosen default branch. With no options selected, the repository stays empty and no commit is created.

README uses the project name and description escaped as plain Markdown text. Licenses are written to root `LICENSE`. The 13 `.gitignore` choices come from [github/gitignore](https://github.com/github/gitignore): Rust, Node.js, Python, Go, Java, C++, Visual Studio/.NET, Dart, Swift, Kotlin, Ruby, Unity, and Android. Assets use a pinned commit, preserve the CC0-1.0 license, and record source URLs and SHA-256 hashes in `provenance.json`.

## License templates

| Template ID | License |
| --- | --- |
| `mit` | MIT |
| `apache-2.0` | Apache License 2.0 |
| `bsd-2-clause` | BSD 2-Clause |
| `bsd-3-clause` | BSD 3-Clause |
| `mpl-2.0` | Mozilla Public License 2.0 |
| `gpl-3.0-only` | GNU GPL v3.0 only |
| `agpl-3.0-only` | GNU AGPL v3.0 only |
| `unlicense` | Unlicense |

MIT and BSD templates fill the current UTC year and copyright holder. The holder defaults to the owning organization name and can be edited in the form. Other licenses retain their canonical text and appendix examples. The legacy `gpl-3.0` ID maps to GPL-3.0-only.

Texts come from pinned [SPDX License List Data v3.27.0](https://github.com/spdx/license-list-data/tree/v3.27.0/text) and are compiled using `include_str!`. Project creation needs no network download. Original assets, source URLs, and SHA-256 hashes live in `crates/gity-projects/templates/` and its `provenance.json`.

## API

`GET /api/v1/repository-templates/licenses` returns available `id`, `name`, and `spdx_id` values. `GET /api/v1/repository-templates/gitignores` returns `.gitignore` template `id` and `name` values. The Web menus consume these two lists and handle loading, errors, and retries independently. Selecting no template remains available.

`POST /api/v1/projects` accepts these initialization fields alongside existing project parameters:

```json
{
  "organization_id": "123456789012345678",
  "name": "Example",
  "key": "example",
  "default_branch": "main",
  "initialize_with_readme": true,
  "license_template": "mit",
  "license_copyright_holder": "Example Contributors",
  "gitignore_template": "rust"
}
```

`initialize_with_readme` defaults to false. Missing, empty, or `none` template values add no file. `license_copyright_holder` applies only to MIT/BSD. Unknown templates or malformed options return 400 before persistence. Creation still requires organization Developer access or higher. Git storage failures compensate the project record and return 503.

Local and Git-node RPC initialization share the same implementation: write one tree and create one root commit without overwriting an existing repository. Nodes retain initialization state until the commit completes. Recovery checks the branch and initial-content fingerprint and refuses mismatched existing commits. Initialization does not schedule CI; existing events and reconciliation update search projections.
