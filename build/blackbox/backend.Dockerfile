FROM golang:1.26-alpine AS build

WORKDIR /src

RUN apk add --no-cache ca-certificates git

COPY go.mod go.sum ./
RUN go mod download

COPY . .

RUN CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o /out/gity-standalone ./cmd/standalone

FROM alpine:3.22

RUN apk add --no-cache ca-certificates git openssh-client tzdata \
    && addgroup -S gity \
    && adduser -S -G gity gity \
    && mkdir -p /var/lib/gity \
    && chown -R gity:gity /var/lib/gity

COPY --from=build /out/gity-standalone /usr/local/bin/gity
RUN chmod 0755 /usr/local/bin/gity

USER gity
WORKDIR /var/lib/gity

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/gity"]
