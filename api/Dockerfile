FROM rust:slim AS build

WORKDIR /app

# Dependency cache layer — copy manifests + migrations first.
# Migrations must be present at compile time for sqlx::migrate!().
COPY Cargo.toml Cargo.lock* ./
COPY migrations ./migrations

# Dummy main so cargo can resolve and cache all dependencies.
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Real source — touch main.rs to bust the Docker layer timestamp cache.
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Runtime image ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/fpnotes-api /usr/local/bin/fpnotes-api

CMD ["fpnotes-api"]
