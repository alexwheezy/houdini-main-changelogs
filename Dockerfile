ARG RUST_VERSION=1.55.0

FROM rust:$RUST_VERSION as build

WORKDIR /directory

COPY ./Cargo.toml ./
COPY src src

RUN cargo build --release

# ----------------------------------------------------------------- #

FROM debian:9-slim

RUN seq 1 8 | xargs -I{} mkdir -p /usr/share/man/man{} && \
  apt update && \
  apt -y install libpq-dev postgresql-client ca-certificates && \
  update-ca-certificates && \
  apt clean

WORKDIR /app

COPY --from=build /directory/target/release/houdnini_main_changelogs ./bot
COPY ./entrypoint.sh ./

CMD ["/app/entrypoint.sh"]
