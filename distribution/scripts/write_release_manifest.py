#!/usr/bin/env python3
import json
import os
import sys


def main() -> int:
    if len(sys.argv) < 4:
        print(
            "usage: write_release_manifest.py <version> <tag> <output-file> [artifact ...]",
            file=sys.stderr,
        )
        return 1

    version = sys.argv[1]
    tag = sys.argv[2]
    output_file = sys.argv[3]
    artifacts = sys.argv[4:]

    payload = {
        "version": version,
        "tag": tag,
        "artifacts": artifacts,
    }

    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    with open(output_file, "w", encoding="utf-8") as fh:
        json.dump(payload, fh, indent=2)
        fh.write("\n")

    print(f"wrote {output_file}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
