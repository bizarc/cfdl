---
id: install-vscode
title: "VS Code and LSP"
slug: "/install/vscode"
---

> This page is generated from `distribution/install-configure.md`.
> Source: https://github.com/bizarc/cfdl/blob/main/distribution/install-configure.md

This guide is for domain experts and model authors who want to use CFDL in VSCode.

Online onboarding docs: `https://bizarc.github.io/cfdl/`

At launch, the extension will be published to the VS Code Marketplace and
Open VSX (install by searching "CFDL"); until then, install from Release
assets as described below.

## What you install

From a GitHub Release, download:

- the VSCode extension package: `cfdl-vscode-<version>.vsix`
- language docs bundle: `cfdl-docs-<version>.tar.gz` (optional, for offline docs)
- packs bundle: `cfdl-packs-<version>.tar.gz` (recommended for pack-enabled models)
- one `cfdl-lsp` binary for your operating system:
  - macOS Apple Silicon: `cfdl-lsp-darwin-arm64`
  - Linux x64: `cfdl-lsp-linux-x64`
  - Windows x64: `cfdl-lsp-windows-x64.exe`

## Step 1: Install the VSCode extension

1. Open VSCode.
2. Open Extensions view.
3. Select `...` and choose **Install from VSIX...**.
4. Pick `cfdl-vscode-<version>.vsix`.

## Step 2: Place the `cfdl-lsp` binary

Store the binary in a stable location on your machine.

Examples:

- macOS/Linux: `~/bin/cfdl-lsp`
- Windows: `C:\\Tools\\cfdl\\cfdl-lsp.exe`

On macOS/Linux, make it executable:

```bash
chmod +x /path/to/cfdl-lsp
```

## Step 3: Configure the extension

In VSCode settings (`settings.json`), set:

```json
{
  "cfdl.serverPath": "/absolute/path/to/cfdl-lsp",
  "cfdl.entryFile": "model.cfdl"
}
```

Optional settings:

- `cfdl.packsPath`: pack directory path
- `cfdl.enableLoweringValidation`: set to `true` or `false`
- `cfdl.trace.server`: `off`, `messages`, or `verbose`

## Step 3.5: Configure packs path (if using packs)

If you downloaded `cfdl-packs-<version>.tar.gz`, extract it to a stable path.

Example:

```bash
mkdir -p ~/.cfdl
tar -xzf cfdl-packs-<version>.tar.gz -C ~/.cfdl
```

Then set:

```json
{
  "cfdl.packsPath": "/absolute/path/to/extracted/packs"
}
```

Note: the VSIX also includes bundled packs/docs, but setting `cfdl.packsPath` to an explicit packs directory is the most predictable production setup.

## Step 4: Start authoring

1. Open your CFDL model folder in VSCode.
2. Open a `.cfdl` file.
3. Use CFDL language features while authoring:
   - diagnostics
   - completions
   - go-to-definition
   - semantic highlighting
   - `CFDL: Apply Pack Template` command

## Troubleshooting

- If features are missing, verify `cfdl.serverPath` points to the correct binary.
- If using packs, verify `cfdl.packsPath` is correct for your model workspace.
- Enable `cfdl.trace.server = verbose` to inspect client/server activity.
