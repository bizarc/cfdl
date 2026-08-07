---
id: install-api-server
title: Run the API server
slug: /docs/install/api-server
generated: none
---

# Run the API server

`cfdl-server` is a self-hostable HTTP API over the compiler and engine —
filesystem-free, embedded packs only.

This page covers running it. For the endpoints it exposes, see
[API server](/docs/api-server).

## Docker

```bash
git clone https://github.com/bizarc/cfdl
cd cfdl
docker build -f crates/cfdl-server/Dockerfile -t cfdl-server .
docker run -p 8080:8080 cfdl-server
```

At launch the image is published as `ghcr.io/bizarc/cfdl-server`.

## From source

```bash
cargo run --release -p cfdl-server
```

## Configuration

- `CFDL_SERVER_ADDR` — bind address (default `0.0.0.0:8080`).
- Built-in limits: 1 MiB request
  body, 10 s request timeout, Monte Carlo trials capped at 1000 (rejected,
  never truncated).

## Verify

```bash
curl localhost:8080/healthz          # {"status":"ok"}
open http://localhost:8080/docs      # Swagger UI
```

Next: [API endpoints and usage](../api-server)
