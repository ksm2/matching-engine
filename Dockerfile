FROM rust:1.65-bullseye AS builder

WORKDIR /usr/src
COPY . /usr/src
RUN cargo build --locked --release


FROM debian:bullseye AS runner

ENV RUST_LOG=info

COPY --from=builder /usr/src/target/release/matching-engine /usr/bin/

ENTRYPOINT ["/usr/bin/matching-engine"]
EXPOSE 3000
