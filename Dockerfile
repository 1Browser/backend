FROM rust:1.81.0 AS builder

WORKDIR /usr/src/1browser

COPY . .

RUN cargo install --path .

FROM debian:bookworm

RUN apt-get update && apt install -y openssl ca-certificates

COPY --from=builder /usr/local/cargo/bin/onebrowser /usr/local/bin/onebrowser

ENTRYPOINT ["onebrowser"]
