---
id: api-server
title: API Server
slug: /docs/api-server
---

# API Server

`cfdl-server` is a small, self-hostable HTTP API over the CFDL compiler and
engine. It is filesystem-free: sources arrive in the request body and packs
come from the embedded registry.

## Endpoints

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/healthz` | — | `{"status":"ok"}` |
| POST | `/v1/compile` | `{files, root_file?}` | IR JSON, or 422 diagnostics |
| POST | `/v1/validate` | `{files, root_file?}` | `{"ok":true}`, or 422 diagnostics |
| POST | `/v1/run` | `{ir? \| files?, root_file?, config?, rate?, pack?}` | Results JSON, or 422/400 |
| GET | `/openapi.json` | — | OpenAPI 3 document |
| GET | `/docs` | — | Swagger UI |

`files` is an in-memory map of root-relative path → source. `run` accepts
either pre-compiled `ir` or `files` to compile first; `pack` applies an
embedded pack's domain metrics.

## Limits

- Request body: 1 MiB (413 beyond).
- Request timeout: 10 seconds.
- Monte Carlo trials capped at 1000 (400 beyond — never silently truncated).

## Run it

See [Run the API Server](/docs/install/api-server) for Docker and from-source
setup, configuration, and verification. Details also live in
`crates/cfdl-server/README.md` in the repository.
