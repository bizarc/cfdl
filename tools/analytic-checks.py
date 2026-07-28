#!/usr/bin/env python3
"""Check the engine against closed-form finance, not against itself.

The benchmark suite diffs each model against a reference implementation. That
catches coding mistakes but not shared assumptions: for a long time both the
engine and every reference placed an annuity's first payment in the period the
instrument was funded, so all eight benchmarks passed while the numbers matched
no bank's model. Two implementations agreeing is not evidence when they were
written from the same misunderstanding.

These checks are different in kind. Each asserts an identity that follows from
the definition of present value, so it holds for any correct implementation and
cannot be satisfied by copying whatever the engine currently does:

  * a par bond discounted at its coupon rate is worth par (NPV = 0);
  * a fully-amortising loan discounted at its own rate is worth its principal;
  * a level annuity matches the closed-form annuity factor
    a(n,i) = (1 - (1+i)^-n) / i;
  * an annuity due is worth exactly (1+i) times the ordinary annuity.

The last one is the direct test of payment timing, and it is the one that was
failing silently.

Usage: python3 tools/analytic-checks.py
"""

from __future__ import annotations

import json
import pathlib
import subprocess
import tempfile

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
CLI = REPO_ROOT / "target" / "debug" / "cfdl"
TOL = 0.01  # dollars


def run_model(source: str, annual_rate: float) -> dict:
    """Compile and run a model, returning its deterministic block."""
    with tempfile.TemporaryDirectory() as tmp:
        d = pathlib.Path(tmp)
        (d / "model.cfdl").write_text(source)
        ir, res = d / "ir.json", d / "results.json"
        for cmd in (
            [str(CLI), "compile", str(d), "--out", str(ir)],
            [str(CLI), "run", str(ir), "--out", str(res), "--rate", str(annual_rate)],
        ):
            done = subprocess.run(cmd, capture_output=True, text=True)
            if done.returncode != 0:
                raise SystemExit(f"model failed:\n{done.stdout}\n{done.stderr}\n{source}")
        return json.loads(res.read_text())["deterministic"]


def npv(block: dict) -> float:
    value = block["metrics"]["model.npv"]
    return value["amount"] if isinstance(value, dict) else value


def monthly_rate(annual: float) -> float:
    return (1.0 + annual) ** (1.0 / 12.0) - 1.0


def annuity_factor(n: int, i: float) -> float:
    """Present value of 1 per period for n periods — the textbook a(n,i)."""
    return (1.0 - (1.0 + i) ** -n) / i


CHECKS: list[tuple[str, callable]] = []


def check(name):
    def wrap(fn):
        CHECKS.append((name, fn))
        return fn
    return wrap


@check("a par bond discounted at its coupon rate is worth par")
def par_bond() -> tuple[float, float]:
    # Bought at 1,000,000, 5% annual coupon, principal returned at maturity.
    # Discounted at 5%, present value must equal the price paid, so NPV = 0.
    src = """version 0.1
model "par-bond"
time calendar monthly from 2026-01 for 61
entity legal investor
stream investor.purchase on entity legal.investor outflow currency USD {
  schedule on 2026-01
  amount = 1000000
}
stream investor.coupon on entity legal.investor inflow currency USD {
  schedule every year from 2026-01 to 2031-01
  amount = 50000
}
stream investor.principal on entity legal.investor inflow currency USD {
  schedule on 2031-01
  amount = 1000000
}
"""
    return npv(run_model(src, 0.05)), 0.0


@check("a level annuity matches the closed-form annuity factor")
def level_annuity() -> tuple[float, float]:
    # 60 monthly payments of 1,000, discounted at 6% annual.
    src = """version 0.1
model "level-annuity"
time calendar monthly from 2026-01 for 61
entity legal holder
stream holder.payment on entity legal.holder inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 1000
}
"""
    i = monthly_rate(0.06)
    return npv(run_model(src, 0.06)), 1000.0 * annuity_factor(60, i)


@check("an annuity due is worth (1+i) times the ordinary annuity")
def annuity_due_ratio() -> tuple[float, float]:
    # The defining relationship between the two conventions, and the identity
    # that fails when payment timing is wrong.
    ordinary = """version 0.1
model "ordinary"
time calendar monthly from 2026-01 for 61
entity legal holder
stream holder.payment on entity legal.holder inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = 1000
}
"""
    due = ordinary.replace(
        "schedule every month from", "schedule every month due from"
    ).replace('model "ordinary"', 'model "due"')
    i = monthly_rate(0.06)
    return npv(run_model(due, 0.06)), npv(run_model(ordinary, 0.06)) * (1.0 + i)


@check("a fully-amortising loan discounted at its own rate is worth its principal")
def amortising_loan() -> tuple[float, float]:
    # 100,000 over 60 months at 6%. Level payments from pmt(); discounted at
    # the loan rate the payments are worth exactly the amount advanced.
    src = """version 0.1
model "amortising-loan"
time calendar monthly from 2026-01 for 61
entity legal lender
stream lender.advance on entity legal.lender outflow currency USD {
  schedule on 2026-01
  amount = 100000
}
stream lender.repayment on entity legal.lender inflow currency USD {
  schedule every month from 2026-01 to 2030-12
  amount = -pmt(0.06 / 12, 60, 100000)
}
"""
    # pmt uses a simple periodic rate, so discount on the same basis.
    return npv(run_model(src, (1.0 + 0.06 / 12.0) ** 12 - 1.0)), 0.0


def main() -> int:
    if not CLI.exists():
        print(f"analytic-checks: {CLI} not found — run `cargo build -p cfdl-cli`")
        return 1

    failures = 0
    for name, fn in CHECKS:
        got, expected = fn()
        ok = abs(got - expected) <= TOL
        print(f"  [{'PASS' if ok else 'FAIL'}] {name}")
        if not ok:
            print(f"         got {got:,.4f}, expected {expected:,.4f}, "
                  f"differ by {abs(got - expected):,.4f}")
            failures += 1

    print()
    if failures:
        print(f"analytic-checks: {failures} of {len(CHECKS)} identities do not hold")
        return 1
    print(f"analytic-checks: OK ({len(CHECKS)} closed-form identities hold)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
