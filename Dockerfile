FROM rust:alpine
RUN apk add --update musl-dev openssl-dev openssl-libs-static g++
COPY . /home/rust/src
WORKDIR /home/rust/src
ENV RUSTFLAGS="-C link-args=-Wl,-Bstatic -C link-args=-lc"
RUN cargo build --release

FROM alpine

COPY --from=0 /home/rust/src/target/release/onchain-scanner /scanner

RUN chmod +x /scanner
ENTRYPOINT ["/scanner"]
