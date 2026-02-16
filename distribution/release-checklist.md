# Release Checklist

Use this checklist before publishing a release tag.

## Pre-Tag Validation

- [ ] `make fmt`
- [ ] `make lint`
- [ ] `make test`
- [ ] `make gold`
- [ ] `npm run lint` in `editors/vscode`

## Artifact Validation

- [ ] `cfdl-lsp-darwin-arm64` present
- [ ] `cfdl-lsp-linux-x64` present
- [ ] `cfdl-lsp-windows-x64.exe` present
- [ ] `cfdl-vscode-<version>.vsix` present
- [ ] `cfdl-docs-<version>.tar.gz` present
- [ ] `cfdl-packs-<version>.tar.gz` present
- [ ] `SHA256SUMS.txt` present and complete
- [ ] `release-manifest-<version>.json` present
- [ ] VSIX contains bundled docs and packs payload

## End-User Documentation Validation

- [ ] `distribution/install-configure.md` matches artifact names and version flow
- [ ] Root `README.md` links to install/configure guide
- [ ] `editors/vscode/README.md` links to install/configure guide
- [ ] Root `README.md` links to language onboarding guide

## Publish

- [ ] Push signed tag `v<version>`
- [ ] Confirm GitHub Release workflow succeeded
- [ ] Confirm all required artifacts are attached to release
