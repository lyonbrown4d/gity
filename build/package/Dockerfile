FROM alpine:3.22

ARG TARGETPLATFORM
ARG BINARY

RUN apk add --no-cache ca-certificates git openssh-client tzdata \
    && addgroup -S gity \
    && adduser -S -G gity gity \
    && mkdir -p /var/lib/gity \
    && chown -R gity:gity /var/lib/gity

COPY ${TARGETPLATFORM}/${BINARY} /usr/local/bin/gity
RUN chmod 0755 /usr/local/bin/gity

USER gity
WORKDIR /var/lib/gity

ENTRYPOINT ["/usr/local/bin/gity"]
