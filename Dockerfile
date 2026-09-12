# ---------- builder ----------
FROM rust:1.90-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Compile query verification is done against the committed .sqlx cache, not
# a live database, so the image can be built with no DB reachable at all.
ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

# ---------- runtime ----------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin fplx

WORKDIR /app

COPY --from=builder /app/target/release/FPLX ./FPLX
COPY migrations ./migrations

USER fplx
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./FPLX"]
