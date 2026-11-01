# Angos and SeaweedFS smoke result

Run on 2026-11-01 against the approved Angos v1.12.4 image index
`sha256:1b90f6720c43eb143f519aec4c7bb721f180e74536634b64912085e464c65abd`
and `chrislusf/seaweedfs:4.51`. The task-owned Seaweed S3 service was on a
private Docker network. Temporary secret override files were mounted read-only;
their contents are not recorded here.

## Result

The first cold Docker pull used Angos's default signed S3 blob redirect and
failed from the host Docker daemon with `502 Bad Gateway`: the signed URL named
the private service DNS name `gity-angos-seaweed-task:8333`, which the daemon
could not resolve on its network. Gity's OCI gateway therefore sends
`X-Angos-No-Redirect: true` to keep blob and manifest streams inside the
private Angos hop.

The repeat ran with `enable_blob_redirect = false` and
`enable_manifest_redirect = false` in the temporary Angos config. S3 used
`multipart_part_size = "5MiB"` and `multipart_uniform_parts = false`.

| Check | Recorded result |
|---|---|
| Angos readiness | HTTP 200, `{"status":"ready"}` |
| Docker push, 12,586,903-byte random OCI layer | Passed; Docker printed `Pushed` and manifest digest `sha256:c4fb912231effe85c7a4483764ada683663d9b46b877abbfe417151fc326cf4f` |
| Cold Docker pull after removing the local image tag | Passed; Docker printed `141f5b89a359: Download complete` and `Downloaded newer image` |
| OCI blob range, bytes 0–127 | HTTP 206; `Content-Range: bytes 0-127/12586903`; response body 128 bytes |

The blob digest in the range request was
`sha256:aa75755594030f56af780a294f0b068e1328977789a92b2363657267f6f6f73c`.
The uploaded layer exceeds the configured 5 MiB multipart threshold. This
smoke exercised Docker's OCI transfer against the actual Angos S3 backend; it
did not exercise the Gity gateway or prove a multi-part API trace inside
SeaweedFS. The gateway's no-redirect header is covered by the Gity OCI unit
fixture.
