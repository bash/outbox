FROM docker.io/rust:1.86 as builder
WORKDIR /usr/local/src/outboxd
COPY . .
RUN cargo install --path crates/outboxd

FROM debian:bookworm-slim
VOLUME /var/lib/outboxd
WORKDIR /var/lib/outboxd
COPY --from=builder /usr/local/cargo/bin/outboxd /usr/local/bin/outboxd
CMD ["outboxd"]
