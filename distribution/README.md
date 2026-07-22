# CFDL Distribution

This folder defines how CFDL is distributed to end users:

- `cfdl` CLI and `cfdl-lsp` binaries for macOS, Linux, and Windows
- VSCode extension package (`.vsix`)
- API server container image (`ghcr.io/bizarc/cfdl-server`)
- Homebrew formula for the CLI
- language docs bundle (`cfdl-docs-<version>.tar.gz`)
- packs bundle (`cfdl-packs-<version>.tar.gz`)
- installation and configuration documentation

Release artifacts are published to GitHub Releases and are not committed to source control.

## Packaging-tool decision (2026-07-22): keep the hand-rolled pipeline

The launch plan floated adopting **cargo-dist**. **Decision: keep the existing
hand-rolled scripts + release workflow.** It already ships four CLI targets,
three LSP targets, the VSIX, and docs/packs tarballs with a checksum manifest;
cargo-dist would replace working, understood scripts with a tool whose main
value (installer generation, self-update receipts) overlaps the Homebrew/VSIX
work we do anyway, and the migration churn lands right before launch. Revisit
post-1.0 if release cadence increases.

## Homebrew (build-only)

`homebrew/cfdl.rb` is a template; `scripts/gen_homebrew.sh <version>
<release-assets-dir> <out.rb>` fills the version, GitHub release URLs, and
per-binary sha256 from the built CLI assets. Verify locally:

```bash
brew audit --formula ./cfdl.rb
brew install --formula ./cfdl.rb
```

**Publishing** (creating/pushing a `bizarc/homebrew-tap` repo) is a separate,
human-approved step — CI never pushes a tap.

## VS Code Marketplace and Open VSX (build-only)

The release workflow builds the platform VSIX. Two registries, both
publish-gated on human approval:

- **VS Code Marketplace**: `npx @vscode/vsce publish` (needs a publisher PAT).
- **Open VSX**: `ovsx` is a dev dependency of `editors/vscode`. Publish with
  `npx ovsx publish cfdl-vscode-<version>.vsix -p <token>` after creating the
  `cfdl` namespace (`npx ovsx create-namespace cfdl -p <token>`).

**License note:** the extension declares `BUSL-1.1` (source-available). Open
VSX and the VS Code Marketplace accept a declared SPDX license; confirm the
BUSL-1.1 terms are acceptable for public listing before publishing. No
publishing happens without explicit human approval.

## Contents

- `artifact-contract.md` - canonical artifact names and formats
- `release-checklist.md` - release gates and validation checklist
- `install-configure.md` - end-user install and configuration guide
- `release-notes-template.md` - standard release note template
- `homebrew/cfdl.rb` - Homebrew formula template (filled by `gen_homebrew.sh`)
- `scripts/` - local helper scripts for packaging
