"""CFDL SDK — compile and run cash-flow models, with pandas result accessors.

Quickstart::

    import cfdl_sdk
    results = cfdl_sdk.run("examples/cre_developer", packs_dir="packs",
                           config="examples/cre_developer/run.base.json")
    results.cashflows()   # wide DataFrame over a PeriodIndex
    results.metrics()     # flat Series of core + domain metrics
"""
from __future__ import annotations

from ._errors import CfdlError, CompileError, Diagnostic, RunError
from ._model import Model, compile, run
from ._native import compile_model, run_ir
from ._results import Results

__all__ = [
    # high-level API
    "compile",
    "run",
    "Model",
    "Results",
    # errors
    "Diagnostic",
    "CfdlError",
    "CompileError",
    "RunError",
    # thin native passthrough (stable, low-level)
    "compile_model",
    "run_ir",
]
