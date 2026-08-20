#!/usr/bin/env python3
"""Check properties the engine must hold whatever a model says.

Every other gate checks an output: goldens match, benchmarks reconcile,
examples compile. These check invariants, and each one exists because its
violation was found by hand first (docs/13 §7.41).

1. CASH PURITY. Streams are the only thing that is cash. A model carrying a
   field, a pack subtotal, an entity rollup and a waterfall must report
   `model.net_cash_flow` equal to the sum of its model streams to the cent,
   `model.total` equal to that series' sum, and `model.npv` equal to the same
   streams discounted at their published offsets. If a field, a subtotal, a
   rollup or a waterfall step ever leaks into cash or valuation, one of the
   three identities breaks. The engine holds this by the collections it
   iterates, not by any name prefix — the comment that once said otherwise
   went stale silently, which is why this is measured.

2. PACK ADDITIVITY. A contract lowers to streams, so any clause the language
   gives a stream must be accepted by contracts, waived deliberately, or
   recorded as a known gap. This is a ratchet on the two parser surfaces:
   a new stream clause fails the gate until someone decides which bucket it
   belongs in, and a known gap that closes fails the gate until it is removed
   from the list.

Usage: python3 tools/invariant-checks.py
"""

from __future__ import annotations

import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
CLI = REPO_ROOT / "target" / "debug" / ("cfdl.exe" if os.name == "nt" else "cfdl")
CENT = 0.01

CASH_PURITY_MODEL = """\
version 0.1
model "invariant-cash-purity"
use pack "credit" version "0.1.0"
time calendar monthly from 2017-01 for 6

entity asset trust : Credit.Asset.LoanPool {
  collateral_type = "auto"
  // A field large enough that leaking into cash is unmissable.
  big_balance init 1000000.0 next prev + 1000000.0
}
entity asset pool : Credit.Asset.LoanPool {
  collateral_type = "auto"
  part of asset.trust
}
entity party investor : Credit.Party.Investor { name = "Investor" }

// A pack contract, so the credit subtotals (cumulative among them) exist.
contract credit.pool_level_pay.one on entity asset.pool {
  term 2017-01..2017-06
  terms { balance = 1000000 rate = 0.12 term_months = 6 cpr = 0 cdr = 0 }
}

stream trust.fee_income on entity asset.trust inflow currency USD {
  schedule every month from 2017-01 to 2017-06
  amount = 2500.0
}

waterfall dist on entity asset.trust {
  schedule every month from 2017-01 to 2017-06
  from available
  pay senior   to party.investor = 100000.0
  pay residual to party.investor = remaining
}
"""

# The model streams above, spelled out. Waterfall steps also publish under
# `stream.` and must NOT be in this list — that exclusion is the test.
MODEL_STREAMS = [
    "stream.credit.pool.interest.one",
    "stream.credit.pool.sched_principal.one",
    "stream.credit.pool.prepay.one",
    "stream.credit.pool.recoveries.one",
    "stream.credit.pool.servicing.one",
    "stream.credit.pool.penalty.one",
    "stream.trust.fee_income",
]
WATERFALL_STEPS = ["stream.dist.senior", "stream.dist.residual"]
ANNUAL_RATE = 0.08


def run_model(source: str) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        d = pathlib.Path(tmp)
        (d / "model.cfdl").write_text(source, encoding="utf-8")
        ir, res = d / "ir.json", d / "results.json"
        packs = str(REPO_ROOT / "packs")
        for cmd in (
            [str(CLI), "compile", str(d), "--out", str(ir), "--packs", packs],
            [str(CLI), "run", str(ir), "--out", str(res), "--rate", str(ANNUAL_RATE),
             "--packs", packs, "--pack", "credit"],
        ):
            done = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
            if done.returncode != 0:
                raise SystemExit(f"invariant model failed:\n{done.stdout}\n{done.stderr}")
        return json.loads(res.read_text(encoding="utf-8"))["deterministic"]


def amounts(entry: dict) -> list[float]:
    return [v["amount"] if isinstance(v, dict) else (v or 0.0)
            for v in entry["values"]]


def check_cash_purity() -> list[str]:
    failures = []
    det = run_model(CASH_PURITY_MODEL)
    series = det["series"]

    missing = [k for k in MODEL_STREAMS + WATERFALL_STEPS if k not in series]
    if missing:
        return [f"cash-purity: expected series absent: {missing}"]

    periods = len(series["model.net_cash_flow"]["values"])
    net = amounts(series["model.net_cash_flow"])
    summed = [0.0] * periods
    for name in MODEL_STREAMS:
        for t, v in enumerate(amounts(series[name])):
            summed[t] += v
    for t in range(periods):
        if abs(net[t] - summed[t]) > CENT:
            failures.append(
                f"cash-purity: period {t}: model.net_cash_flow {net[t]:.4f} != "
                f"sum of model streams {summed[t]:.4f} — something besides a "
                f"stream entered cash"
            )

    total = det["metrics"]["model.total"]["amount"]
    if abs(total - sum(net)) > CENT:
        failures.append(
            f"cash-purity: model.total {total:.4f} != sum of net cash {sum(net):.4f}"
        )

    # NPV, rebuilt from nothing but the model streams and their published
    # offsets. If anything else entered valuation the figure moves.
    ppy = 12.0
    per = (1.0 + ANNUAL_RATE) ** (1.0 / ppy) - 1.0
    npv = 0.0
    for name in MODEL_STREAMS:
        offset = series[name].get("offset", 0.0) or 0.0
        scale = (1.0 + per) ** (-offset)
        for t, v in enumerate(amounts(series[name])):
            npv += scale * v / (1.0 + per) ** t
    reported = det["metrics"]["model.npv"]["amount"]
    if abs(reported - npv) > CENT:
        failures.append(
            f"cash-purity: model.npv {reported:.4f} != streams discounted at "
            f"their offsets {npv:.4f} — something besides a stream entered "
            f"valuation"
        )
    return failures


# --- pack additivity -------------------------------------------------------

PARSER = REPO_ROOT / "crates" / "cfdl-parser" / "src" / "lib.rs"

# Stream clauses a contract deliberately does not take: the pack's lowering
# rules own the per-stream answer, and a term on the contract feeds them.
WAIVED = {
    "name": "both have one",
    "attached_entity": "a contract has subject_entity",
    "span": "parser bookkeeping",
    "direction": "per lowered stream; the pack's lowering rule declares it",
    "currency": "per lowered stream; the pack's lowering rule declares it",
    "category": "per lowered stream; the pack's lowering rule declares it",
    "amount": "per lowered stream; the lowering rule's amount_expr is the amount",
    "schedule": "a contract's term plus the rule's schedule_kind own the cadence",
}

# Stream clauses a contract SHOULD take and does not yet. docs/13 §7.40.
# Closing one of these must remove it here, or the gate fails.
KNOWN_GAPS = {
    "active_when": "docs/13 §7.40 instance 1 — a contract cannot be gated",
    "active_in_states": "docs/13 §7.40 instance 1 — a contract cannot be gated",
}


def struct_fields(source: str, name: str) -> set[str]:
    m = re.search(rf"pub struct {name} \{{(.*?)^\}}", source, re.S | re.M)
    if not m:
        raise SystemExit(f"pack-additivity: struct {name} not found in parser")
    return set(re.findall(r"^    pub (\w+):", m.group(1), re.M))


def check_pack_additivity() -> list[str]:
    failures = []
    source = PARSER.read_text(encoding="utf-8")
    stream = struct_fields(source, "StreamStmt")
    contract = struct_fields(source, "ContractStmt")

    for clause in sorted(stream):
        accepted = clause in contract
        waived = clause in WAIVED
        known = clause in KNOWN_GAPS
        if accepted and known:
            failures.append(
                f"pack-additivity: contracts now accept '{clause}' — remove it "
                f"from KNOWN_GAPS and close its backlog entry"
            )
        elif not accepted and not waived and not known:
            failures.append(
                f"pack-additivity: StreamStmt gained '{clause}' and ContractStmt "
                f"does not accept it. Add it to contracts, waive it with a "
                f"reason, or record it as a known gap with a backlog entry."
            )
    for clause in sorted(KNOWN_GAPS):
        if clause not in stream:
            failures.append(
                f"pack-additivity: KNOWN_GAPS names '{clause}', which StreamStmt "
                f"no longer declares — update the list"
            )
    return failures


def main() -> int:
    if not CLI.exists():
        print(f"invariant-checks: CFDL binary not found at {CLI}; build cfdl-cli first")
        return 2
    failures = check_cash_purity() + check_pack_additivity()
    if failures:
        for f in failures:
            print(f"[invariants][FAIL] {f}")
        return 1
    print(
        "invariant-checks: OK (cash purity holds across a field, a cumulative "
        "subtotal, an entity rollup and a waterfall; the contract surface "
        "accounts for every stream clause)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
