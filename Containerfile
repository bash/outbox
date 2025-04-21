FROM docker.io/rust:1.86 as builder
WORKDIR /usr/local/src/outboxd
COPY . .
RUN cargo install --path crates/outboxd

FROM debian:bookworm-slim
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y msmtp && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/outboxd /usr/local/bin/outboxd
RUN mkdir -p /etc/outboxd
RUN mkdir -p /var/log/outboxd
VOLUME /var/lib/outboxd
WORKDIR /var/lib/outboxd
CMD ["outboxd", "-C", "/etc/outboxd/msmtprc", "--logfile", "/var/log/outboxd/msmtp.log"]
