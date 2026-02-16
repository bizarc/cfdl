---
id: troubleshooting
title: Troubleshooting
---

# Troubleshooting

## Missing language features in VSCode

- Verify `cfdl.serverPath` points to the correct `cfdl-lsp` binary.
- Set `cfdl.trace.server` to `verbose` for debugging.

## Pack diagnostics unexpectedly failing

- Confirm `cfdl.packsPath` points to an extracted packs directory.
- Confirm model `use pack` ID and version match available pack manifests.

## Compile failures on imports

- Ensure import paths are relative to the importing file.
- Avoid import cycles and root-escape paths.

## Diagnostic code references

- `docs/diagnostics_spec.md`
