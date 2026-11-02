<div lang="en">

[简体中文](../../zh/reference/http-api.md) · [English](http-api.md)

# HTTP API contract

Utoipa macros generate contracts from actual types and routes. Only development builds explicitly enabling `api-docs` serve `/docs` and `/openapi.json`; releases exclude documentation endpoints and Swagger assets. See [API transports](api-transports.md) for GraphQL, MCP, configuration, and boundaries.

Project operations require the appropriate user or project token. The server enforces permissions and scopes. Public JSON entity IDs are strings. See [API and permission examples](../api-examples.md).

TSV files in `contracts/` are engineering check data, not a second authoritative public contract. Update documentation macros with route and DTO changes, and inspect the generated result.

</div>
