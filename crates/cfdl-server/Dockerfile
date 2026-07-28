# CFDL API server — multi-stage build to a distroless runtime image.
# Build context is the repository root:
#   docker build -f crates/cfdl-server/Dockerfile -t ghcr.io/bizarc/cfdl-server .
# Publishing to ghcr.io is a human-approved step.

FROM rust:1-slim AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p cfdl-server

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /src/target/release/cfdl-server /usr/local/bin/cfdl-server
ENV CFDL_SERVER_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/cfdl-server"]
