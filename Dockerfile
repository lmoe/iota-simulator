FROM postgres:16-bookworm as db-builder

# Environment variables for PostgreSQL
ENV POSTGRES_PASSWORD=postgres
ENV POSTGRES_USER=postgres
ENV POSTGRES_DB=postgres
ENV PGDATA=/var/lib/postgresql/data

# Initialize PostgreSQL data directory during build
RUN mkdir -p "$PGDATA" && \
    chown -R postgres:postgres "$PGDATA" && \
    gosu postgres initdb && \
    # Modify postgresql.conf for faster startup
    echo "fsync = off" >> "$PGDATA/postgresql.conf" && \
    echo "synchronous_commit = off" >> "$PGDATA/postgresql.conf" && \
    echo "full_page_writes = off" >> "$PGDATA/postgresql.conf" && \
    # Start PostgreSQL temporarily to initialize the database
    gosu postgres pg_ctl -D "$PGDATA" start && \
    # Wait for PostgreSQL to start
    until gosu postgres pg_isready; do sleep 1; done && \
    # Run your database initialization commands here
    # gosu postgres psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -c 'CREATE TABLE example (id SERIAL PRIMARY KEY, name TEXT);' && \
    # Stop PostgreSQL
    gosu postgres pg_ctl -D "$PGDATA" stop

# Rust builder stage
FROM rustlang/rust:nightly-bookworm as rust-builder

WORKDIR /usr/src/app

# Copy your Rust project files
COPY build.rs .
COPY Cargo.lock .
COPY Cargo.toml .
COPY src ./src

# Install PostgreSQL 16 and required dependencies
RUN apt-get update && \
    apt-get install -y \
        curl \
        gnupg2 \
        lsb-release \
        clang \
        libclang-dev && \
    curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /usr/share/keyrings/postgresql-keyring.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/postgresql-keyring.gpg] http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/postgresql.list && \
    apt-get update && \
    apt-get install -y postgresql-16 libpq-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*


# Build the Rust application in release mode
RUN cargo build --release  --package iota-l1-simulator  --bin iota-l1-simulator

# Final image
FROM postgres:16
#RUN apk add gcompat
ENV PGDATA=/var/lib/postgresql/data

# Copy the pre-initialized database
COPY --from=db-builder --chown=postgres:postgres /var/lib/postgresql/data $PGDATA

RUN chmod -R 700 $PGDATA

# Copy the compiled Rust binary
COPY --from=rust-builder /usr/src/app/target/release/iota-l1-simulator /usr/local/bin/app

# Create startup script
RUN echo '#!/bin/bash\n\
gosu postgres postgres -D $PGDATA & \n\
until gosu postgres pg_isready; do\n\
    sleep 1\n\
done\n\
/usr/local/bin/app' > /usr/local/bin/start.sh && \
    chmod +x /usr/local/bin/start.sh

CMD ["/usr/local/bin/start.sh"]