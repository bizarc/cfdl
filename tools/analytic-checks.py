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
    return periodic_rate(annual, 12)


def periodic_rate(annual: float, ppy: int) -> float:
    """The per-period rate the engine discounts at on a `ppy`-period grid."""
    return (1.0 + annual) ** (1.0 / ppy) - 1.0


def annuity_factor(n: int, i: float) -> float:
    """Present value of 1 per period for n periods — the textbook a(n,i)."""
    return (1.0 - (1.0 + i) ** -n) / i



def value_of(expr: str, scale: float = 1.0) -> float:
    """Evaluate one CFDL expression through the engine.

    A single-period model at a zero discount rate has NPV equal to its one
    amount, so this is the shortest path from an expression to a number that
    the real evaluator produced. `scale` lifts small quantities — an SMM is
    ~0.004 — into a range where the module's dollar tolerance is meaningful.
    """
    src = f"""version 0.1
model "expr"
time calendar monthly from 2026-01 for 1
entity legal e
stream e.v on entity legal.e inflow currency USD {{
  schedule every month from 2026-01 to 2026-01
  amount = ({expr}) * {scale}
}}
"""
    return npv(run_model(src, 0.0))


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
  schedule every year from 2026-01 to 2030-12
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


@check("a term deferring to an input disperses under Monte Carlo")
def term_input_disperses() -> tuple[float, float]:
    """Sampling must reach a driver referenced from a contract term.

    Terms were once baked into lowered expressions as literals, so a trial
    sampled a variable the expression did not contain and every trial returned
    the same number. The golden suite cannot guard this: a long Monte Carlo run
    over a pack expression is not bit-identical across platforms. What matters
    is that the spread is real, so that is what is asserted.
    """
    src = """version 0.1
model "term-dispersion"
time calendar monthly from 2026-01 for 12
entity project plant
assume driver ~ Normal(mean=1000, stdev=100, clip=[600, 1400])
stream plant.revenue on entity project.plant inflow currency USD {
  schedule every month from 2026-01 to 2026-12
  amount = inputs.driver
}
run monte_carlo trials 400 seed 20260728
"""
    with tempfile.TemporaryDirectory() as tmp:
        d = pathlib.Path(tmp)
        (d / "model.cfdl").write_text(src)
        ir, res = d / "ir.json", d / "results.json"
        for cmd in (
            [str(CLI), "compile", str(d), "--out", str(ir)],
            [str(CLI), "run", str(ir), "--out", str(res), "--rate", "0.1"],
        ):
            done = subprocess.run(cmd, capture_output=True, text=True)
            if done.returncode != 0:
                raise SystemExit(f"model failed:\n{done.stdout}\n{done.stderr}")
        block = json.loads(res.read_text()).get("monte_carlo") or {}
    stdev = ((block.get("metrics") or {}).get("model.npv") or {}).get("stdev")
    stdev = stdev["amount"] if isinstance(stdev, dict) else (stdev or 0.0)
    # Any real spread proves the driver reached the expression; compare against
    # a threshold rather than a value so the check is platform-independent.
    return (1.0 if stdev > 1.0 else 0.0), 1.0



# Until the cadence work, every model in the repo — fixtures, benchmarks, and
# every check above — was monthly. The engine's discount_offset and annuity-due
# placement had therefore never run on a quarterly or annual grid at all. These
# assert the same closed forms there, so that a pack dividing by
# periods-per-year is composing with discounting that is known to be right.


@check("a level annuity matches a(n,i) on quarterly and annual grids")
def level_annuity_off_monthly() -> tuple[float, float]:
    worst = (0.0, 0.0)
    for calendar, interval, ppy, periods in [
        ("quarterly", "quarter", 4, 20),
        ("annual", "year", 1, 5),
    ]:
        src = f"""version 0.1
model "level-annuity-{calendar}"
time calendar {calendar} from 2026-01 for {periods + 1}
entity legal holder
stream holder.payment on entity legal.holder inflow currency USD {{
  schedule every {interval} from 2026-01 to {2026 + (periods * 12 // ppy - 12) // 12}-{((periods - 1) * 12 // ppy) % 12 + 1:02d}
  amount = 1000
}}
"""
        i = periodic_rate(0.06, ppy)
        got = npv(run_model(src, 0.06))
        expected = 1000.0 * annuity_factor(periods, i)
        if abs(got - expected) > abs(worst[0] - worst[1]):
            worst = (got, expected)
    return worst


@check("an annuity due is worth (1+i) times the ordinary annuity, off monthly")
def annuity_due_ratio_off_monthly() -> tuple[float, float]:
    worst = (0.0, 0.0)
    for calendar, interval, ppy, periods in [
        ("quarterly", "quarter", 4, 20),
        ("annual", "year", 1, 5),
    ]:
        last_month = ((periods - 1) * 12 // ppy) % 12 + 1
        last_year = 2026 + (periods - 1) * 12 // ppy // 12
        ordinary = f"""version 0.1
model "ordinary-{calendar}"
time calendar {calendar} from 2026-01 for {periods + 1}
entity legal holder
stream holder.payment on entity legal.holder inflow currency USD {{
  schedule every {interval} from 2026-01 to {last_year}-{last_month:02d}
  amount = 1000
}}
"""
        due = ordinary.replace(
            f"schedule every {interval} from", f"schedule every {interval} due from"
        ).replace(f'model "ordinary-{calendar}"', f'model "due-{calendar}"')
        i = periodic_rate(0.06, ppy)
        got = npv(run_model(due, 0.06))
        expected = npv(run_model(ordinary, 0.06)) * (1.0 + i)
        if abs(got - expected) > abs(worst[0] - worst[1]):
            worst = (got, expected)
    return worst


# SIFMA, Standard Formulas for the Analysis of Mortgage-Backed Securities
# (Uniform Practices Manual, Chapter SF). The definitional source for CPR, SMM
# and PSA — the conventions the credit pack's own term names come from. Free to
# download, NOT redistributable, so these assert its published figures without
# reproducing the document.
#
# These belong here rather than in a benchmark because they are identities: they
# follow from the definitions and hold for any correct implementation.


@check("SIFMA: (1 - SMM)^12 = 1 - CPR, and cpr_to_periodic inverts it at any cadence")
def sifma_smm_cpr_identity() -> tuple[float, float]:
    worst = (0.0, 0.0)
    for cpr in ["0.02", "0.06", "0.1136151282838708", "0.25"]:
        # Compounding the periodic rate over a year must recover the annual one.
        for ppy in [1, 4, 12, 365]:
            got = value_of(
                f"1 - pow(1 - cpr_to_periodic({cpr}, {ppy}), {ppy})", 1e6
            )
            expected = float(cpr) * 1e6
            if abs(got - expected) > abs(worst[0] - worst[1]):
                worst = (got, expected)
    return worst


@check("SIFMA: the published 6/89 GNMA worked example, every intermediate")
def sifma_worked_example() -> tuple[float, float]:
    # A Ginnie Mae I 9.0% pass-through issued 3/1/88, 359 months remaining at
    # 6/1/89, 9.5% gross coupon. Chapter SF section 2 works it end to end and
    # publishes every step, so each is asserted rather than only the answer.
    r = "0.095 / 12"
    bal = lambda k: f"(1 - pow(1 + {r}, -{k})) / (1 - pow(1 + {r}, -359))"
    fsched = f"0.85150625 * ({bal(343)}) / ({bal(344)})"
    smm = f"100 * (({fsched}) - 0.84732282) / ({fsched})"
    cpr = f"100 * (1 - pow(1 - ({smm}) / 100, 12))"

    # Scales are set by the SOURCE's precision, not by how tight we could make
    # them. The factors are published to eight decimals, so at scale 1e6 the
    # module's 0.01 tolerance asserts +/- 1e-8 — exactly the published figure,
    # and no further. Asserting past a source's own rounding is not rigour.
    cases = [
        (bal(344), 0.99213300, 1e6),   # BAL1
        (bal(343), 0.99157471, 1e6),   # BAL2
        (fsched, 0.85102709, 1e6),     # scheduled-only factor
        (f"0.85150625 - ({fsched})", 0.00047916, 1e6),        # amortization
        (f"({fsched}) - 0.84732282", 0.00370427, 1e6),        # prepayments
        (smm, 0.435270, 1e3),          # SMM, published to 6 decimals as a %
        (cpr, 5.1000, 1e3),            # CPR %
        # 6/89 is month 17 of the underlying loans, so the implied speed is
        # CPR / min(0.2 * MONTH, 6.0).
        # PSA is published as "150.00%" — two decimals — so scale 1 asserts
        # +/- 0.01 percentage points, which is that figure and no more.
        (f"100 * ({cpr}) / min(0.2 * 17, 6.0)", 150.00, 1.0),
    ]
    worst = (0.0, 0.0)
    for expr, expected, scale in cases:
        got = value_of(expr, scale)
        want = expected * scale
        if abs(got - want) > abs(worst[0] - worst[1]):
            worst = (got, want)
    return worst


@check("SIFMA: the PSA ramp, 0.2% CPR per month to 6.0% at month 30 and flat after")
def sifma_psa_curve() -> tuple[float, float]:
    # CPR = min(PSA/100 * 0.2 * max(1, min(MONTH, 30)), 100). Closed-form
    # today; the pack cannot yet USE it, because a ramped hazard makes the pool
    # factor a cumulative product rather than pow(k, p) — see the backlog. This
    # pins the shape so the ramp work starts from a checked curve.
    psa_cpr = lambda speed, month: (
        f"min({speed} / 100 * 0.2 * max(1, min({month}, 30)), 100)"
    )
    cases = [
        (100, 1, 0.2), (100, 17, 3.4), (100, 29, 5.8),
        (100, 30, 6.0), (100, 60, 6.0),
        (150, 17, 5.1),      # the worked example above, forwards
        (150, 30, 9.0), (200, 30, 12.0),
    ]
    worst = (0.0, 0.0)
    for speed, month, expected in cases:
        got = value_of(psa_cpr(speed, month), 1e4)
        want = expected * 1e4
        if abs(got - want) > abs(worst[0] - worst[1]):
            worst = (got, want)
    return worst

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
