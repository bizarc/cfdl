# CFDL VSCode Extension

This extension provides CFDL language features (diagnostics, completion, go-to-definition, semantic highlighting, and pack-template commands) through `cfdl-lsp`.

For end-user installation and configuration, see `../../distribution/install-configure.md`.

## Prerequisites

- VSCode 1.85+
- Node 20+ (for extension build)
- A built `cfdl-lsp` binary

Build the server (from repo root):

```bash
cargo build -p cfdl-lsp
```

## Fastest development/test loop

1. Open `editors/vscode` as the VSCode workspace folder.
2. Run `npm install` (first time) and `npm run compile`.
3. Start **Run and Debug** -> **Run CFDL Extension**.
4. In the Extension Development Host window, open a `.cfdl` file and test language features.

## Server discovery order

The extension resolves `cfdl-lsp` in this order:

1. `cfdl.serverPath` setting (absolute path)
2. `cfdl-lsp` found on `PATH`
3. `${workspaceFolder}/target/debug/cfdl-lsp`
4. `${workspaceFolder}/target/release/cfdl-lsp`

If features do not activate, set `cfdl.serverPath` explicitly.

## End-user workflow

Use the distribution guide for non-development usage:

- install `.vsix`
- install platform `cfdl-lsp` binary
- set `cfdl.serverPath`
- author CFDL models in VSCode
