---
id: install-cli
title: Install the CLI
slug: /install/cli
---

# Install the CLI

The `cfdl` binary compiles, validates, and runs models:
`cfdl compile · validate · parse · run · pack list`.

## Homebrew (at launch)

```bash
brew install cfdl
```

## Prebuilt binaries (GitHub Releases)

Download the binary for your platform from a release, place it on your
`PATH`, and (macOS/Linux) make it executable:

- macOS Apple Silicon: `cfdl-darwin-arm64`
- Linux x64: `cfdl-linux-x64`
- Windows x64: `cfdl-windows-x64.exe`

```bash
chmod +x ~/bin/cfdl
```

## From source (current pre-launch path)

With a Rust toolchain (the repo pins one via `rust-toolchain.toml`):

```bash
git clone https://github.com/bizarc/cfdl
cd cfdl
cargo build --release -p cfdl-cli
# binary at target/release/cfdl
```

## Getting the packs

The CLI resolves `use pack` against a packs directory passed with
`--packs`. Domain packs (energy, cre, credit, opco) ship as a
`cfdl-packs-<version>.tar.gz` release asset; from a checkout, use the
repo's `packs/` directory:

```bash
cfdl compile my-model --packs packs --out my-model/ir.json
```

## Verify

```bash
cfdl --json validate examples/language_tutorial/minimal_model
cfdl pack list --path packs
```

Next: [Getting Started](../getting-started) · [CLI usage in the Language Guide](../language-guide)
