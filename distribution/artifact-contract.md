# Artifact Contract

This document defines the canonical release artifact set for each CFDL SDK version tag (for example `v0.1.0`).

## Required Artifacts

- `cfdl-lsp-darwin-arm64`
- `cfdl-lsp-linux-x64`
- `cfdl-lsp-windows-x64.exe`
- `cfdl-vscode-<version>.vsix`
- `cfdl-docs-<version>.tar.gz`
- `cfdl-packs-<version>.tar.gz`
- `SHA256SUMS.txt`
- `release-manifest-<version>.json`

## Naming Rules

- `<version>` maps to the extension version in `editors/vscode/package.json`.
- Binary names are platform-stable and do not include version text.
- The VSIX includes version text so users can identify package version before install.
- Docs and packs bundles include version text and are published as release assets.

## Integrity

- `SHA256SUMS.txt` must include all binaries, the VSIX, docs bundle, and packs bundle.
- `release-manifest-<version>.json` must enumerate artifact filenames and source git tag.

## VSIX Bundled Content Contract

The VSIX package must contain:

- `extension/bundled/docs/LANGUAGE_GUIDE.md`
- `extension/bundled/docs/install-configure.md`
- `extension/bundled/packs/cre/pack.toml`
- `extension/bundled/packs/opco/pack.toml`
