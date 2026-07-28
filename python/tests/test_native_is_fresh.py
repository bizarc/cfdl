"""Guards against running the SDK's tests on a stale native module.

The editable install compiles `cfdl_sdk._native` from the Rust crates once;
nothing rebuilds it when the engine changes. A stale module silently passes
tests against old behaviour — this exact situation was found during a
pre-launch surface check, where the installed SDK still accepted a model the
engine had learned to reject days earlier.
"""

from __future__ import annotations

import pathlib

import pytest

import cfdl_sdk

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]


def _newest_engine_source_mtime() -> tuple[float, pathlib.Path]:
    """Most recently modified file that is compiled into the native module."""
    candidates: list[tuple[float, pathlib.Path]] = []
    for pattern in ("crates/**/*.rs", "crates/**/Cargo.toml", "packs/**/*.toml"):
        for path in REPO_ROOT.glob(pattern):
            if "target" in path.parts:
                continue
            candidates.append((path.stat().st_mtime, path))
    return max(candidates)


def _native_module_path() -> pathlib.Path | None:
    native = getattr(cfdl_sdk, "_native", None)
    if native is None:  # pragma: no cover - import shape guard
        from cfdl_sdk import _native as native  # type: ignore[no-redef]
    file = getattr(native, "__file__", None)
    return pathlib.Path(file) if file else None


def test_native_module_is_not_older_than_the_engine_sources() -> None:
    """Fail loudly rather than testing yesterday's compiler."""
    module_path = _native_module_path()
    if module_path is None or not module_path.exists():
        pytest.skip("native module path unavailable (packaged install)")

    module_mtime = module_path.stat().st_mtime
    source_mtime, source_path = _newest_engine_source_mtime()

    assert module_mtime >= source_mtime, (
        f"The installed native module is older than the engine sources.\n"
        f"  module: {module_path} ({module_mtime:.0f})\n"
        f"  source: {source_path.relative_to(REPO_ROOT)} ({source_mtime:.0f})\n"
        f"Rebuild it before trusting these results:\n"
        f"  make py-develop        # inside a virtualenv\n"
        f"  # or: cd python && maturin build --release && "
        f"pip install --force-reinstall --no-deps <wheel>"
    )


def test_native_module_carries_current_pack_validations() -> None:
    """A behavioural freshness check that survives packaged installs.

    Timestamps are unavailable for a wheel-installed module, so assert on a
    capability the current engine has: pack domain validations reject an
    out-of-range prepayment rate.
    """
    model = REPO_ROOT / "fixtures" / "invalid" / "credit_invalid_rates"
    if not model.exists():
        pytest.skip("fixture unavailable")

    with pytest.raises(cfdl_sdk.CompileError) as excinfo:
        cfdl_sdk.run(str(model), packs_dir=str(REPO_ROOT / "packs"))

    codes = {diagnostic.code for diagnostic in excinfo.value.diagnostics}
    assert "E9010_CREDIT_INVALID_CPR" in codes, (
        "The native module did not apply the credit pack's validations; it is "
        "probably built from older sources. Rebuild with `make py-develop`."
    )
