## CFDL VSCode Distribution

## Highlights in v0.2.3

- Contract lowering now binds stream ownership to contract subject (`on entity ...`) instead of implicitly relying on parent/first-entity behavior.
- CRE and OpCo lowering rules were updated to use subject-based ownership defaults.
- Added compatibility coverage for contracts that omit `on entity`, plus new fixtures and goldens for:
  - explicit non-first-entity subject behavior
  - unresolved contract subject diagnostics
- Updated example models and docs to reflect current contract-vs-stream authoring guidance.

This release includes:

- `cfdl-lsp` binaries for macOS, Linux, and Windows
- CFDL VSCode extension package (`.vsix`)
- language docs bundle (`cfdl-docs-<version>.tar.gz`)
- packs bundle (`cfdl-packs-<version>.tar.gz`)
- checksums (`SHA256SUMS.txt`)

## Install

Follow `distribution/install-configure.md` for end-user setup instructions.
Documentation: https://cfdl.dev

## Artifacts

- `cfdl-lsp-darwin-arm64`
- `cfdl-lsp-darwin-universal` (Intel + Apple Silicon)
- `cfdl-lsp-linux-x64`
- `cfdl-lsp-windows-x64.exe`
- `cfdl-vscode-<version>.vsix`
- `SHA256SUMS.txt`
