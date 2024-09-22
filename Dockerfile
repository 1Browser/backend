FROM rust:1.81.0 AS builder

WORKDIR /usr/src/1browser

COPY . .

RUN cargo install --path .

FROM debian:bookworm

COPY --from=builder /usr/local/cargo/bin/onebrowser /usr/local/bin/onebrowser

ENTRYPOINT ["onebrowser"]
