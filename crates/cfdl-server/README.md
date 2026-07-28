# cfdl-server

A small, filesystem-free HTTP API for the CFDL compiler and engine, over the
**embedded** pack registry. Source-available (BUSL-1.1); not published.

## Endpoints

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/healthz` | — | `{"status":"ok"}` |
| POST | `/v1/compile` | `{files, root_file?}` | IR JSON, or 422 diagnostics |
| POST | `/v1/validate` | `{files, root_file?}` | `{"ok":true}`, or 422 diagnostics |
| POST | `/v1/run` | `{ir? \| files?, root_file?, config?, rate?, pack?}` | Results JSON, or 422/400 |
| GET | `/openapi.json` | — | OpenAPI 3 document |
| GET | `/docs` | — | Swagger UI |

`files` is an in-memory map of root-relative path → source; there is no
filesystem access. `pack` applies that pack's domain metrics (embedded packs
only). `run` accepts either pre-compiled `ir` or `files` to compile first.

## Limits

- Request body: **1 MiB** (`413` beyond).
- Request timeout: **10 s**.
- Monte Carlo trials: capped at **1000** (`400` beyond — never silently
  truncated).

See `src/limits.rs`.

## Run it

```bash
cargo run -p cfdl-server        # binds CFDL_SERVER_ADDR (default 0.0.0.0:8080)
curl localhost:8080/healthz
```

## Container

```bash
docker build -f crates/cfdl-server/Dockerfile -t ghcr.io/bizarc/cfdl-server .
docker run -p 8080:8080 ghcr.io/bizarc/cfdl-server
```

Publishing the image to `ghcr.io` is a human-approved step; CI builds and
saves the image as an artifact but never pushes.
