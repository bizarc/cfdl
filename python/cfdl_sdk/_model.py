"""High-level compile/run API: :func:`compile`, :func:`run`, :class:`Model`."""
from __future__ import annotations

import json
from pathlib import Path

from . import _native
from ._errors import raise_from_native
from ._results import Results

PathLike = "str | Path"


def _as_str(value) -> str | None:
    return None if value is None else str(value)


def _config_to_json(config) -> str | None:
    """Normalize a config argument to a string the native layer accepts.

    Accepts a dict (serialized to JSON), a path to a JSON file (passed through
    as a string; the native layer detects a file), or a raw JSON string.
    """
    if config is None:
        return None
    if isinstance(config, dict):
        return json.dumps(config)
    return str(config)


def _detect_pack(model_dir: Path) -> str | None:
    """Fixture convention: a ``pack`` file names the active pack."""
    pack_file = model_dir / "pack"
    if pack_file.is_file():
        return pack_file.read_text(encoding="utf-8").strip() or None
    return None


class Model:
    """A compiled CFDL model (IR), ready to run."""

    def __init__(
        self,
        ir_json: str,
        *,
        model_dir: Path | None = None,
        packs_dir: str | None = None,
    ):
        self.ir_json = ir_json
        self._model_dir = model_dir
        self._packs_dir = packs_dir
        self._ir: dict | None = None

    @property
    def ir(self) -> dict:
        if self._ir is None:
            self._ir = json.loads(self.ir_json)
        return self._ir

    def run(
        self,
        *,
        config=None,
        rate: float = 0.0,
        as_of: str | None = None,
        pack: str | None = None,
        packs_dir=None,
    ) -> Results:
        """Run this model's IR. ``packs_dir``/``pack`` default to the values
        detected at compile time."""
        resolved_packs = _as_str(packs_dir) if packs_dir is not None else self._packs_dir
        resolved_pack = pack
        if resolved_pack is None and self._model_dir is not None:
            resolved_pack = _detect_pack(self._model_dir)
        try:
            results_json = _native.run_ir(
                self.ir_json,
                packs_dir=resolved_packs,
                config_json=_config_to_json(config),
                rate=rate,
                as_of=as_of,
                pack=resolved_pack,
            )
        except RuntimeError as exc:
            raise raise_from_native(exc) from None
        return Results.from_json(results_json)


def compile(model_dir, *, packs_dir=None) -> Model:
    """Compile a model directory to IR. Raises :class:`CompileError`."""
    model_path = Path(model_dir)
    packs = _as_str(packs_dir)
    try:
        ir_json = _native.compile_model(str(model_path), packs_dir=packs)
    except RuntimeError as exc:
        raise raise_from_native(exc) from None
    return Model(ir_json, model_dir=model_path, packs_dir=packs)


def run(
    model_dir,
    *,
    packs_dir=None,
    config=None,
    rate: float = 0.0,
    as_of: str | None = None,
    pack: str | None = None,
) -> Results:
    """Compile and run a model directory in one call."""
    model = compile(model_dir, packs_dir=packs_dir)
    return model.run(config=config, rate=rate, as_of=as_of, pack=pack)
