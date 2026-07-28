# CFDL API server — multi-stage build to a distroless runtime image.
# Build context is the repository root:
#   docker build -f crates/cfdl-server/Dockerfile -t ghcr.io/bizarc/cfdl-server .
# Publishing to ghcr.io is a human-approved step.

FROM rust:1-slim AS builder
# utoipa-swagger-ui's build script downloads the Swagger UI bundle at compile
# time and shells out to curl when its reqwest feature is off. rust:1-slim
# ships neither curl nor CA certificates, so the build script exits 101.
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN cargo build --release -p cfdl-server

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/cfdl-server /usr/local/bin/cfdl-server
ENV CFDL_SERVER_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cfdl-server"]
