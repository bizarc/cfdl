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


def wal(block: dict) -> float:
    return block["metrics"]["model.wal_years"]


def payback(block: dict) -> float:
    return block["metrics"]["model.payback_years"]


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


CHECKS: list[tuple[str, callable, float]] = []


def check(name, tol=TOL):
    """Register an identity. `tol` defaults to the dollar tolerance.

    WAL and payback identities are in YEARS, where the dollar tolerance is
    meaningless — a one-period slip is 0.0833 on a monthly grid. Those pass
    WAL_TOL instead, which is set by the metric's own rounding.
    """
    def wrap(fn):
        CHECKS.append((name, fn, tol))
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


# Market-standard MBS prepayment conventions — CPR, SMM and the standard
# prepayment curve — which the credit pack's term names come from. Checked for
# parity against the published industry reference; the figures below are cited
# as facts, and no external document is reproduced.
#
# These belong here rather than in a benchmark because they are identities: they
# follow from the definitions and hold for any correct implementation.


@check("MBS: (1 - SMM)^12 = 1 - CPR, and cpr_to_periodic inverts it at any cadence")
def mbs_smm_cpr_identity() -> tuple[float, float]:
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


@check("MBS: a published GNMA pool-factor back-out, every intermediate")
def mbs_pool_factor_backout() -> tuple[float, float]:
    # A Ginnie Mae I 9.0% pass-through issued 3/1/88, 359 months remaining at
    # 6/1/89, 9.5% gross coupon. The industry reference works this end to end
    # and gives every step, so each is asserted rather than only the answer.
    r = "0.095 / 12"
    bal = lambda k: f"(1 - pow(1 + {r}, -{k})) / (1 - pow(1 + {r}, -359))"
    fsched = f"0.85150625 * ({bal(343)}) / ({bal(344)})"
    smm = f"100 * (({fsched}) - 0.84732282) / ({fsched})"
    cpr = f"100 * (1 - pow(1 - ({smm}) / 100, 12))"

    # Scales are set by the reference's precision, not by how tight we could make
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


@check("MBS: the standard prepayment curve, 0.2% CPR/month to 6.0% at month 30")
def mbs_standard_prepayment_curve() -> tuple[float, float]:
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

# ---------------------------------------------------------------------------
# Weighted average life and payback: the TIME AXIS.
#
# These are here rather than in a benchmark for the reason the file exists. A
# reference generator written alongside the engine shares its conventions, so
# the three credit benchmarks asserted WAL against their own restatement of the
# same off-by-one for as long as they existed. An identity does not care what
# the engine thinks: a bullet's weighted average life IS its term, by the
# definition of the words.
#
# The convention being pinned: a flow's time is (period + offset) / ppy, the
# same axis discounting uses (docs/12_payment_timing.md). An ordinary annuity's
# first monthly payment is therefore at 1/12 year, not 0.
# ---------------------------------------------------------------------------

# Years. Set by round_amount's 6-decimal output, not by slack: 1/12 is
# published as 0.083333, so a difference of two rounded metrics can be off by
# 1e-6 with nothing wrong. The error being hunted is a full period — 0.0833 on
# a monthly grid, 1.0 on an annual one — so this is still five orders of
# magnitude clear of it.
WAL_TOL = 1e-6


def _single_flow(calendar: str, periods: int, schedule: str) -> str:
    return f"""version 0.1
model "wal-probe"
time calendar {calendar} from 2026-01 for {periods}
entity legal investor
stream investor.bullet on entity legal.investor inflow currency USD {{
  schedule {schedule}
  amount = 1000000
}}
"""


@check("WAL: a bullet's weighted average life is its term", WAL_TOL)
def wal_bullet_equals_term() -> tuple[float, float]:
    # One payment, at the end of month 60. Its average life is 5 years by the
    # definition of an average over a single point. This is the check that
    # fails on the pre-fix engine, which reports 4.9167 — the whole defect,
    # in one line.
    src = _single_flow("monthly", 61, "every month from 2030-12 to 2030-12")
    return wal(run_model(src, 0.08)), 5.0


@check("WAL: an ordinary annuity is exactly one period later than an annuity due", WAL_TOL)
def wal_due_vs_ordinary() -> tuple[float, float]:
    # Same cash, same periods, different placement in each period. The whole
    # difference between them is one period, so the WALs differ by 1/ppy and
    # by nothing else.
    ordinary = wal(run_model(_single_flow(
        "monthly", 24, "every month from 2026-01 to 2027-12"), 0.08))
    due = wal(run_model(_single_flow(
        "monthly", 24, "every month due from 2026-01 to 2027-12"), 0.08))
    return ordinary - due, 1.0 / 12.0


@check("WAL: `mid` sits exactly halfway between due and ordinary", WAL_TOL)
def wal_mid_is_halfway() -> tuple[float, float]:
    # Pins discount_offset's 0.5 branch through the WAL path. Annual grid, so
    # a half period is half a year and the assertion has room to be wrong.
    mid = wal(run_model(_single_flow(
        "annual", 5, "every year mid from 2026-01 to 2030-01"), 0.08))
    due = wal(run_model(_single_flow(
        "annual", 5, "every year due from 2026-01 to 2030-01"), 0.08))
    return mid - due, 0.5


@check("WAL: the same deal has the same average life on any calendar", WAL_TOL)
def wal_calendar_invariant() -> tuple[float, float]:
    # One payment five years out, expressed three ways. WAL is a real duration,
    # so the grid it is measured on cannot change it. Catches any ppy slip in
    # the (idx + offset) / ppy conversion — the class of error the cadence work
    # found in discount_offset's own day rule.
    monthly = wal(run_model(_single_flow(
        "monthly", 61, "every month from 2030-12 to 2030-12"), 0.08))
    quarterly = wal(run_model(_single_flow(
        "quarterly", 21, "every quarter from 2030-10 to 2030-10"), 0.08))
    annual = wal(run_model(_single_flow(
        "annual", 6, "every year from 2030-01 to 2030-01"), 0.08))
    return max(abs(monthly - 5.0), abs(quarterly - 5.0), abs(annual - 5.0)), 0.0


@check("WAL: lies between the first and last inflow instants", WAL_TOL)
def wal_is_bounded() -> tuple[float, float]:
    # Cheap, and it catches sign and denominator slips that the exact
    # identities above could in principle both miss.
    src = """version 0.1
model "wal-bounds"
time calendar monthly from 2026-01 for 37
entity legal investor
stream investor.early on entity legal.investor inflow currency USD {
  schedule every month from 2026-01 to 2026-01
  amount = 400000
}
stream investor.late on entity legal.investor inflow currency USD {
  schedule every month from 2028-12 to 2028-12
  amount = 600000
}
"""
    got = wal(run_model(src, 0.08))
    # First inflow at the end of period 0 = 1/12; last at the end of period 35.
    inside = (1.0 / 12.0) <= got <= (36.0 / 12.0)
    return 1.0 if inside else 0.0, 1.0


@check("payback: a single inflow repays on its own date, not a period early", WAL_TOL)
def payback_uses_the_same_axis() -> tuple[float, float]:
    # An outflow at t=0 and one inflow at the end of month 12. The model turns
    # cash-positive when that inflow lands, which is 1 year out. Reporting
    # 0.9167 would be the same off-by-one as WAL, from the same raw index.
    src = """version 0.1
model "payback-probe"
time calendar monthly from 2026-01 for 13
entity legal investor
stream investor.outlay on entity legal.investor outflow currency USD {
  schedule on 2026-01
  amount = 1000000
}
stream investor.repay on entity legal.investor inflow currency USD {
  schedule every month from 2026-12 to 2026-12
  amount = 1000000
}
"""
    return payback(run_model(src, 0.08)), 1.0


def main() -> int:
    if not CLI.exists():
        print(f"analytic-checks: {CLI} not found — run `cargo build -p cfdl-cli`")
        return 1

    failures = 0
    for name, fn, tol in CHECKS:
        got, expected = fn()
        ok = abs(got - expected) <= tol
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
