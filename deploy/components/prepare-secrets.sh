#!/bin/sh
set -eu

: "${SEAWEED_ACCESS_KEY_ID:?Load the deployment .env before running this script}"
: "${SEAWEED_SECRET_KEY:?Load the deployment .env before running this script}"
: "${GIT_NODE_1_PEER_TOKEN:?Load the deployment .env before running this script}"
: "${GIT_NODE_2_PEER_TOKEN:?Load the deployment .env before running this script}"
GITY_DOCKERHUB_USERNAME=${GITY_DOCKERHUB_USERNAME:-}
GITY_DOCKERHUB_TOKEN=${GITY_DOCKERHUB_TOKEN:-}
for value in "$SEAWEED_ACCESS_KEY_ID" "$SEAWEED_SECRET_KEY" "$GIT_NODE_1_PEER_TOKEN" "$GIT_NODE_2_PEER_TOKEN" "$GITY_DOCKERHUB_USERNAME" "$GITY_DOCKERHUB_TOKEN"; do
  case "$value" in *[!A-Za-z0-9_-]*) echo 'Secret values may contain only letters, digits, _ and -' >&2; exit 2 ;; esac
done
for token in "$GIT_NODE_1_PEER_TOKEN" "$GIT_NODE_2_PEER_TOKEN"; do
  [ "${#token}" -ge 32 ] || { echo 'Git node peer tokens must be at least 32 characters' >&2; exit 2; }
done

default_secrets_dir="$(CDPATH= cd -- "$(dirname -- "$0")/secrets" && pwd)"
secrets_dir="${GITY_SECRETS_DIR:-$default_secrets_dir}"
mkdir -p "$secrets_dir"
umask 077
printf '%s' "$GIT_NODE_1_PEER_TOKEN" > "$secrets_dir/git-node-1-token"
printf '%s' "$GIT_NODE_2_PEER_TOKEN" > "$secrets_dir/git-node-2-token"
cat > "$secrets_dir/seaweed-s3.json" <<JSON
{
  "identities": [
    {
      "name": "gity-compose",
      "credentials": [
        {"accessKey": "$SEAWEED_ACCESS_KEY_ID", "secretKey": "$SEAWEED_SECRET_KEY"}
      ],
      "actions": ["Read", "Write", "List", "Tagging", "Admin"]
    }
  ]
}
JSON
cat > "$secrets_dir/angos-s3.toml" <<TOML
[blob_store.s3]
access_key_id = "$SEAWEED_ACCESS_KEY_ID"
secret_key = "$SEAWEED_SECRET_KEY"
TOML
if [ -n "$GITY_DOCKERHUB_USERNAME" ] || [ -n "$GITY_DOCKERHUB_TOKEN" ]; then
  [ -n "$GITY_DOCKERHUB_USERNAME" ] && [ -n "$GITY_DOCKERHUB_TOKEN" ] || {
    echo 'Set both Docker Hub username and token, or leave both empty' >&2
    exit 2
  }
  for value in "$GITY_DOCKERHUB_USERNAME" "$GITY_DOCKERHUB_TOKEN"; do
    case "$value" in *[!A-Za-z0-9._-]*) echo 'Docker Hub credentials may contain only letters, digits, ., _ and -' >&2; exit 2 ;; esac
  done
  cat > "$secrets_dir/angos-mirror.toml" <<TOML
[[repository."mirror-dockerhub".upstream]]
url = "https://registry-1.docker.io"
username = "$GITY_DOCKERHUB_USERNAME"
password = "$GITY_DOCKERHUB_TOKEN"
TOML
else
  cat > "$secrets_dir/angos-mirror.toml" <<'TOML'
[[repository."mirror-dockerhub".upstream]]
url = "https://registry-1.docker.io"
TOML
fi
chmod 0444 "$secrets_dir/git-node-1-token" "$secrets_dir/git-node-2-token" "$secrets_dir/seaweed-s3.json" "$secrets_dir/angos-s3.toml" "$secrets_dir/angos-mirror.toml"
printf 'Generated Git node credentials and authenticated SeaweedFS S3 config in %s\n' "$secrets_dir"
