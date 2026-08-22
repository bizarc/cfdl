#!/usr/bin/env python3
"""Decide whether a change needs the three-platform end-to-end.

Reads changed paths on stdin, one per line, and prints `code=true|false` in
GitHub output format. `false` means every changed path is prose: it cannot
affect a build, a test, a golden or a notebook, so the matrix would only add
latency. It does NOT mean the change is unchecked — every gate runs on every
change, in the `gates` job, which has no condition.

Fails safe: an empty list, or anything unrecognised, is `true`.

This replaced `dorny/paths-filter`. That action OR-matches the patterns in a
filter, so a list of `'**'` followed by `'!docs/**'` exclusions returned true
for every file — `'**'` matched and the negations never subtracted. The bug is
silent in exactly the direction that looks fine (everything runs), which is why
it needs a test rather than a pattern list.
"""

import sys

# docs/schemas/** is deliberately NOT prose: a published schema is a contract,
# and code is validated against it.
PROSE = (
    lambda p: p.startswith("docs/") and not p.startswith("docs/schemas/"),
    lambda p: p.startswith("research/"),
    lambda p: p.endswith(".md") or p.endswith(".mdx"),
    lambda p: p in ("LICENSE", "NOTICE"),
)


def is_prose(path: str) -> bool:
    return any(rule(path) for rule in PROSE)


def classify(paths: list[str]) -> bool:
    """True when the end-to-end matrix should run."""
    if not paths:
        return True
    return any(not is_prose(p) for p in paths)


# The bug this replaced was silent in the safe-looking direction, so the rules
# are pinned here and checked in CI before they are used.
SELFTEST = (
    (["docs/13_feature_backlog.md", "docs/26_lessons_learned.md"], False),
    (["research/notes.md"], False),
    (["packs/cre/README.md"], False),
    ([".github/workflows/ci.yml", "makefile"], True),
    (["docs/schemas/run.schema.json"], True),
    (["docs/13_feature_backlog.md", "crates/cfdl-engine/src/lib.rs"], True),
    (["site/src/app/page.tsx"], True),
    ([], True),
)


def selftest() -> int:
    failed = 0
    for paths, want in SELFTEST:
        got = classify(paths)
        if got != want:
            failed += 1
            print(f"FAIL {paths}: want code={want}, got code={got}", file=sys.stderr)
    if failed:
        print(f"ci-classify-paths: {failed} case(s) FAILED", file=sys.stderr)
        return 1
    print(f"ci-classify-paths: OK ({len(SELFTEST)} cases)", file=sys.stderr)
    return 0


def main() -> None:
    if "--selftest" in sys.argv:
        sys.exit(selftest())
    paths = [line.strip() for line in sys.stdin if line.strip()]
    code = classify(paths)
    for path in paths:
        print(f"{'code ' if not is_prose(path) else 'prose'}  {path}", file=sys.stderr)
    print(
        f"\n{len(paths)} changed path(s); matrix {'RUNS' if code else 'is skipped'}",
        file=sys.stderr,
    )
    print(f"code={'true' if code else 'false'}")


if __name__ == "__main__":
    main()
