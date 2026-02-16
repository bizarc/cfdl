#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def read_checksums(path: Path) -> set[str]:
    names: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        parts = line.split()
        if len(parts) < 2:
            continue
        names.add(parts[-1])
    return names


def read_manifest_artifacts(path: Path) -> set[str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    artifacts = data.get("artifacts", [])
    if not isinstance(artifacts, list):
        return set()
    names: set[str] = set()
    for item in artifacts:
        if isinstance(item, str):
            for name in item.split():
                names.add(name)
        else:
            names.add(str(item))
    return names


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: verify_release_assets.py <version> <release-assets-dir>",
            file=sys.stderr,
        )
        return 1

    version = sys.argv[1]
    assets_dir = Path(sys.argv[2]).resolve()
    if not assets_dir.exists() or not assets_dir.is_dir():
        print(f"assets directory not found: {assets_dir}", file=sys.stderr)
        return 1

    required_assets = {
        "cfdl-lsp-darwin-arm64",
        "cfdl-lsp-linux-x64",
        "cfdl-lsp-windows-x64.exe",
        f"cfdl-vscode-{version}.vsix",
        f"cfdl-docs-{version}.tar.gz",
        f"cfdl-packs-{version}.tar.gz",
        "SHA256SUMS.txt",
        f"release-manifest-{version}.json",
    }

    existing_assets = {p.name for p in assets_dir.iterdir() if p.is_file()}
    missing_assets = sorted(required_assets - existing_assets)
    if missing_assets:
        print("missing required assets:", file=sys.stderr)
        for name in missing_assets:
            print(f"  - {name}", file=sys.stderr)
        return 1

    checksums_file = assets_dir / "SHA256SUMS.txt"
    checksummed = read_checksums(checksums_file)
    required_in_checksums = {
        "cfdl-lsp-darwin-arm64",
        "cfdl-lsp-linux-x64",
        "cfdl-lsp-windows-x64.exe",
        f"cfdl-vscode-{version}.vsix",
        f"cfdl-docs-{version}.tar.gz",
        f"cfdl-packs-{version}.tar.gz",
    }
    missing_checksums = sorted(required_in_checksums - checksummed)
    if missing_checksums:
        print("missing required checksum entries:", file=sys.stderr)
        for name in missing_checksums:
            print(f"  - {name}", file=sys.stderr)
        return 1

    manifest_file = assets_dir / f"release-manifest-{version}.json"
    manifest_artifacts = read_manifest_artifacts(manifest_file)
    required_in_manifest = required_assets - {manifest_file.name}
    missing_manifest_entries = sorted(required_in_manifest - manifest_artifacts)
    if missing_manifest_entries:
        print("missing release-manifest artifact entries:", file=sys.stderr)
        for name in missing_manifest_entries:
            print(f"  - {name}", file=sys.stderr)
        return 1

    print("release assets verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
