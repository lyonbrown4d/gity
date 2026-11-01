#!/bin/sh
set -eu

default_tls_dir="$(CDPATH= cd -- "$(dirname -- "$0")/tls" && pwd)"
tls_dir="${GITY_TLS_DIR:-$default_tls_dir}"
umask 077
mkdir -p "$tls_dir"

openssl genpkey -algorithm ED25519 -out "$tls_dir/ca.key"
openssl req -x509 -new -key "$tls_dir/ca.key" -out "$tls_dir/ca.crt" \
  -days 3650 -subj "/CN=Gity local deployment CA" \
  -addext "basicConstraints=critical,CA:TRUE" \
  -addext "keyUsage=critical,keyCertSign,cRLSign"

issue_certificate() {
  name="$1"
  san="$2"
  openssl genpkey -algorithm ED25519 -out "$tls_dir/$name.key"
  openssl req -new -key "$tls_dir/$name.key" -out "$tls_dir/$name.csr" \
    -subj "/CN=$name"
  printf 'basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature\nsubjectAltName=%s\nextendedKeyUsage=serverAuth,clientAuth\n' "$san" \
    > "$tls_dir/$name.ext"
  openssl x509 -req -in "$tls_dir/$name.csr" -CA "$tls_dir/ca.crt" \
    -CAkey "$tls_dir/ca.key" -CAcreateserial -out "$tls_dir/$name.crt" \
    -days 825 -extfile "$tls_dir/$name.ext"
  rm "$tls_dir/$name.csr" "$tls_dir/$name.ext"
}

issue_certificate api-server 'DNS:api,DNS:localhost,IP:127.0.0.1'
issue_certificate git-node-1-server 'DNS:git-node,DNS:localhost,IP:127.0.0.1'
issue_certificate git-node-2-server 'DNS:git-node-2,DNS:localhost,IP:127.0.0.1'
issue_certificate api-client 'DNS:api-client'
issue_certificate git-client 'DNS:git-client'
issue_certificate worker-client 'DNS:worker-client'
issue_certificate client 'DNS:gity-compose-client'

chmod 600 "$tls_dir"/*.key
chmod 644 "$tls_dir"/*.crt
printf 'Generated local CA and service certificates in %s\n' "$tls_dir"
printf 'This CA is for local Compose use; rotate it before production deployment.\n'
