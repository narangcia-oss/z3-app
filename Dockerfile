# Stage 1: Build Rust
FROM rust:1.88 AS rust-builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -f src/main.rs

COPY . .

RUN cargo build --release

# Stage 2: Build Tailwind CSS
FROM node AS node-builder
WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .
RUN npm run build:css

# Stage 3: Final image
FROM debian:bookworm-slim AS final
WORKDIR /app

RUN useradd -m appuser

COPY --from=rust-builder /app/target/release/z3-app ./backend

COPY --from=node-builder /app/static/tailwind.css ./static/tailwind.css
COPY --from=node-builder /app/static/htmx.min.js ./static/htmx.min.js
COPY --from=node-builder /app/static/favicon.png ./static/favicon.png

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libpq5 \
  && rm -rf /var/lib/apt/lists/*

USER appuser

EXPOSE 3000

CMD ["./backend"]
