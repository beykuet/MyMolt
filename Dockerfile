# Builder (using musl for static binary)
FROM clux/muslrust:stable AS builder
WORKDIR /volume
COPY . .
RUN cargo build --release --bin zeroclaw

# Runtime (Scratch - 0MB overhead)
FROM scratch
COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/zeroclaw /zeroclaw
# Copy SSL certs for HTTPS requests
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
ENTRYPOINT ["/zeroclaw"]
CMD ["start"]
