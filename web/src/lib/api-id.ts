/**
 * Orval types `int64` path parameters as `number`, but Gity serializes entity IDs
 * as decimal strings and snowflake IDs exceed `Number.MAX_SAFE_INTEGER`
 * (2^53-1). Converting them with `Number()` would silently round the ID and hit
 * the wrong resource (usually a 404).
 *
 * Orval interpolates path parameters with template literals, so keeping the
 * decimal string at runtime produces the exact URL the backend expects. The
 * cast only satisfies the generated `number` signature.
 */
export const toApiPathId = (id: string): number => {
  const normalized = id.trim();
  if (!/^\d+$/.test(normalized)) {
    throw new Error(`Invalid API path id: ${id}`);
  }
  return normalized as unknown as number;
};

const rawJson = (JSON as unknown as { rawJSON?: (value: string) => unknown }).rawJSON;

/**
 * JSON request bodies that contain `int64` IDs have the same precision problem:
 * `Number(id)` rounds, and sending a quoted string fails `Vec<i64>`
 * deserialization. `JSON.rawJSON` keeps the exact decimal digits while still
 * emitting an (unquoted) JSON number for `JSON.stringify`.
 *
 * Prefer fixing the OpenAPI schema/backend to accept decimal strings for request
 * body IDs; use this only for generated `number` fields that must stay exact.
 */
export const toRawJsonId = (id: string): number => {
  const normalized = id.trim();
  if (!/^\d+$/.test(normalized)) {
    throw new Error(`Invalid API body id: ${id}`);
  }
  if (!rawJson) {
    throw new Error("JSON.rawJSON is required to send exact int64 identifiers");
  }
  return rawJson(normalized) as number;
};

