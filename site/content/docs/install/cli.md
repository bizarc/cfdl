---
id: install-cli
title: Install the CLI
slug: /docs/install/cli
generated: none
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

The four domain packs — energy, cre, credit and opco — are built into the
binary, so `use pack` resolves with no flag and no download:

```bash
cfdl compile my-model --out my-model/ir.json
```

Pass `--packs <dir>` to use your own packs instead. A directory containing
packs is authoritative: the CLI will not silently fall back to the built-in
copies if it does not hold the pack your model asks for, so a mistyped path
fails rather than quietly compiling against a different pack than you meant.

The packs also ship as a `cfdl-packs-<version>.tar.gz` release asset if you
want to read or fork one.

## Verify

```bash
cfdl --json validate examples/language_tutorial/minimal_model
cfdl pack list --path packs
```

Next: [Getting Started](../getting-started) · [CLI usage in the Language Guide](../language-guide)
