# Distributed Git Layer (Backend Plan)

This document defines how Gity should evolve from single-node repository storage to distributed repository serving.

## Goals

- Keep current HTTP Git protocol behavior (`upload-pack` / `receive-pack`) compatible with standard Git clients.
- Support horizontal scale by routing repositories across multiple storage nodes.
- Keep control-plane metadata in DB, data-plane Git objects on storage nodes.
- Allow gradual rollout: single-node mode first, then multi-node routing, then replication.

## Target Layers

- `apps/standalone`: control plane + API gateway for metadata and auth.
- `crates/repository`: metadata persistence (organization/repository/access/placement metadata).
- `crates/git`: Git protocol and local storage primitives.
- `apps/git-node` (future): dedicated data-plane node for Git RPC and object storage.

## Phase Plan

1. Placement-aware routing
- Add repository placement metadata (which node owns a repo).
- `standalone` resolves repository owner node before serving Git endpoints.
- If repo is remote, return redirect/proxy to owner node.

2. Dedicated git-node service
- Split Git RPC execution from control plane.
- `standalone` handles auth + policy and forwards signed internal requests to git-node.

3. Replication model
- Start with async replication (primary + followers).
- Push always targets primary.
- Fetch can read from follower after lag check.

4. Failure handling
- Add repository failover procedure (promote follower to primary).
- Persist replication health and last applied revision in metadata DB.

## Data Model Additions (Suggested)

- `repository_placements`
  - `repository_id`
  - `primary_node_id`
  - `strategy` (`single`, `primary_replica`)
  - `updated_at`

- `repository_replicas`
  - `repository_id`
  - `node_id`
  - `role` (`primary`, `replica`)
  - `sync_state` (`healthy`, `lagging`, `offline`)
  - `last_seen_at`

## API/Internal Contracts (Suggested)

- Control plane to data plane:
  - `POST /internal/git/{owner}/{repo}/upload-pack`
  - `POST /internal/git/{owner}/{repo}/receive-pack`
  - Signed internal token with repo + operation + expiry

- Placement service interface:
  - `resolve_primary(owner, repo) -> node`
  - `resolve_read_nodes(owner, repo) -> [node]`

## Migration Strategy

1. Keep current local mode as default.
2. Introduce placement metadata with all existing repos bound to local node.
3. Enable remote routing for selected repos.
4. Introduce replication for selected repos only.

## Non-Goals (Current Stage)

- Geo-distributed consistency guarantees.
- Multi-primary writes for one repository.
- Cross-region conflict resolution.
