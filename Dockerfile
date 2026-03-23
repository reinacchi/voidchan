# Stage 1: Build the Rust application and SQLx CLI
FROM rust:1.88-slim-bullseye AS builder

ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

WORKDIR /usr/src

# Copy the entire repository
COPY . .

# Install sqlx-cli so migrations can run in the runtime container
RUN cargo install sqlx-cli --no-default-features --features rustls,mysql --locked

# Build the release version
RUN cargo build --release

# Stage 2: Create the runtime image
FROM debian:bullseye-slim

ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary, sqlx CLI, and migrations
COPY --from=builder /usr/src/target/release/voidchan /usr/local/bin/voidchan
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /usr/src/migrations ./migrations

# Run migrations first, then start the application
CMD ["sh", "-c", "sqlx migrate run && /usr/local/bin/voidchan"]
