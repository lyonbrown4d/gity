<div lang="en">

[简体中文](../../zh/deployment/oci.md) · [English](oci.md)

# OCI registry deployment

The public OCI API is served by the Gity Rust gateway. Angos is a private
storage and distribution engine bound to loopback in a single-process install,
or to a private container network in a split install. Do not publish the Angos
port or Web UI. The upstream image is pinned in
[`build/package/angos-engine.md`](../reference/angos-engine.md).

The gateway runs the platform's opaque-token validation and project ACL checks
for every request, including hits served from the local cache. The Angos
listener is not an alternate public authentication service. Cross-repository
mount requests require pull access on the source and push access on the target.
The gateway removes the client `Authorization` header before proxying, so
credentials configured for an upstream registry are never taken from a Gity
client session.

The TOML examples show the Angos filesystem and SeaweedFS S3 settings and
pull-through repository entries. Store upstream credentials and S3 credentials
in a mode-0600 generated file from the secret provider; do not put values in the
Gity config or repository. Angos supports mergeable config files, which lets
the base file stay in deployment source control while a secret provider mounts
the override file. Angos v1.12.4's S3 client uses path-style endpoint URLs and
does not expose a `force_path_style` setting.

Mirror mode is read-only at the Gity gateway. Its typed configuration must list
registry aliases, exact HTTPS authorities allowed for registry and token
service traffic, and secret references. The OCI client transport checks each
request and redirect against that host allowlist, rejects HTTP downgrade, and
removes Authorization across origins. Clients cannot select arbitrary hosts
or forward their credentials. Before every mirrored manifest request, Gity
resolves the configured upstream reference and sends Angos the returned digest;
this also applies to mutable version tags. Upstream 401, 403, or transport
failures stop before Angos sees the request, so a cached stale tag cannot
conceal an upstream denial or outage. Local project ACL checks still run for
every cache hit. The resolver uses the upstream OCI Distribution client and
its Gity transport adapter; see
[`build/package/oci-client.md`](../reference/oci-client.md).

Gity uses storage keys derived from stable project IDs so renaming a user-facing
namespace does not move objects. The platform authorization adapter owns the
mapping from public repository path to the stable Angos repository key.

Use SeaweedFS through Angos's standard S3 interface in integrated environments.
Before declaring that combination supported, verify push and pull, multipart
uploads, range requests, cache hit/miss, mutable tag refresh, offline behavior,
and unauthorized cache hits against the exact pinned Angos release. No OCI
black-box pass is claimed until those runs are recorded.

</div>
