FROM rust:1-alpine AS builder
WORKDIR /app
COPY . .
RUN apk --update add cmake make musl-dev pkgconfig && \
	cargo build --release

FROM alpine:3 AS runtime
COPY --from=builder /app/target/release/fancy-tree /usr/local/bin/fancy-tree

ENTRYPOINT ["/usr/local/bin/fancy-tree"]
