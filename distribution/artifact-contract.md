# Artifact Contract

This document defines the canonical release artifact set for each CFDL SDK version tag (for example `v0.1.0`).

## Required Artifacts

- `cfdl-lsp-darwin-arm64`
- `cfdl-lsp-linux-x64`
- `cfdl-lsp-windows-x64.exe`
- `cfdl-vscode-<version>.vsix`
- `SHA256SUMS.txt`
- `release-manifest-<version>.json`

## Naming Rules

- `<version>` maps to the extension version in `editors/vscode/package.json`.
- Binary names are platform-stable and do not include version text.
- The VSIX includes version text so users can identify package version before install.

## Integrity

- `SHA256SUMS.txt` must include all binaries and the VSIX.
- `release-manifest-<version>.json` must enumerate artifact filenames and source git tag.
