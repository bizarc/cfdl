---
id: install-index
title: Install & Setup
slug: /docs/install
---

# Install & Setup

CFDL runs the same compiler and engine everywhere — pick the surface that
matches how you work.

| You want to… | Surface | Setup |
|---|---|---|
| Try the language right now | [Playground](/playground) | [none](/docs/install/playground) |
| Model in files, run in CI | CLI | [Install the CLI](/docs/install/cli) |
| Analyze results in pandas / notebooks | Python SDK | [Install for Python](/docs/install/python) |
| Call CFDL from another service | API server | [Run the server](/docs/install/api-server) |
| Author models with diagnostics + hover | VS Code + LSP | [VS Code and LSP](/docs/install/vscode) |

## Pre-launch note

CFDL is pre-1.0 and the repository is currently private. Until launch,
installs come **from source or from GitHub Release assets**; the pages here
mark the channels that open at launch (Homebrew, PyPI, VS Code Marketplace,
`ghcr.io`) as *at launch*.

## Versions and compatibility

The CLI, Python SDK, WASM playground, and API server embed the same
compiler and engine, so a given release produces byte-identical IR and
Results across all of them. The language/IR spec is v0.1; interfaces may
change until 1.0 freezes the IR and Results schemas.
