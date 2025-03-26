FROM rust:1.71 AS builder
WORKDIR /app
COPY . .

RUN cargo build --release

FROM ubuntu:latest
RUN apt-get update && apt-get install -y libssl-dev pkg-config wget
RUN wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2_amd64.deb
RUN dpkg -i libssl1.1_1.1.1f-1ubuntu2_amd64.deb
RUN apt-get install -y ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/server-bin ~/server
COPY --from=builder /app/migrations ~/migrations
COPY --from=builder /app/packages ~/packages
ENTRYPOINT ["~/server"]

