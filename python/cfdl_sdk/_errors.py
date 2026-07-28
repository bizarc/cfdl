"""Structured error types for the CFDL SDK.

The native module raises ``RuntimeError`` whose message is either a JSON array
of compile diagnostics or a plain engine/run error string. The Python wrapper
(:mod:`cfdl_sdk._model`) catches those and re-raises the typed exceptions here.
"""
from __future__ import annotations

import json
from dataclasses import dataclass, field


@dataclass(frozen=True)
class Diagnostic:
    """A single compile diagnostic (see docs/08_diagnostics.md)."""

    code: str
    severity: str
    message: str
    file: str | None = None
    span: dict | None = None
    hint: str | None = None
    notes: tuple[str, ...] = ()

    @classmethod
    def from_dict(cls, raw: dict) -> "Diagnostic":
        return cls(
            code=raw.get("code", ""),
            severity=raw.get("severity", "error"),
            message=raw.get("message", ""),
            file=raw.get("file"),
            span=raw.get("span"),
            hint=raw.get("hint"),
            notes=tuple(raw.get("notes", []) or ()),
        )

    def __str__(self) -> str:
        loc = ""
        if self.file:
            line = (self.span or {}).get("start_line")
            loc = f" ({self.file}:{line})" if line is not None else f" ({self.file})"
        return f"{self.code}: {self.message}{loc}"


class CfdlError(Exception):
    """Base class for all CFDL SDK errors."""


class CompileError(CfdlError):
    """Compilation failed. Carries the structured diagnostics."""

    def __init__(self, diagnostics: list[Diagnostic]):
        self.diagnostics = diagnostics
        shown = "; ".join(str(d) for d in diagnostics[:5])
        extra = "" if len(diagnostics) <= 5 else f" (+{len(diagnostics) - 5} more)"
        super().__init__(f"compilation failed: {shown}{extra}")


class RunError(CfdlError):
    """The engine or run-config parsing failed (message is a plain string)."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(message)


def raise_from_native(exc: Exception) -> "CfdlError":
    """Translate a native ``RuntimeError`` into a typed CFDL error.

    A JSON-array payload is a list of compile diagnostics; anything else is a
    run-time error string.
    """
    payload = str(exc)
    try:
        parsed = json.loads(payload)
    except (json.JSONDecodeError, ValueError):
        return RunError(payload)
    if isinstance(parsed, list):
        return CompileError([Diagnostic.from_dict(d) for d in parsed])
    return RunError(payload)
