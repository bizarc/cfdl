# agent-eval — the harness that grades authoring agents

The benchmark suite becomes the grader (docs/32 Phase 3). An agent under
test receives a task, drives whatever loop it likes — the intended one is
the `cfdl-mcp` compile → run → diff → explain loop — and returns final CFDL
sources. The harness scores them with the same comparison `make bench` uses,
so the eval and the benchmark suite cannot grade differently.

## Tiers

| tier | task | scored |
|---|---|---|
| `repair` | a minimal failing model + its structured diagnostics (`fixtures/invalid/` + `gold/diag/`, 70 tasks) | compiles |
| `transcribe` | a case's CASE.md and permitted reference material — never `expected.csv` — plus its run configuration (42 public tasks) | compiles / runs / matches, partial credit by asserted column and metric |
| `extend` | an existing model + a change request, graded by targeted assertions (`tasks/extend/*.toml`; empty until assertions with independent derivations exist) | compiles / runs / matches assertions |

## Agents

- `--agent replay` — the scripted agent: returns the known-good sources.
  Must score 100% on repair + transcribe; the runner exits nonzero otherwise.
  This separates harness bugs from model failures.
- `--agent cmd:<command>` — task JSON on stdin, `{"files": {...}}` on stdout.
- `--agent <http url>` — the task JSON is POSTed; same response shape. The
  provider-agnostic seam for real agents.

## Runs

```
python3 tools/agent-eval/runner.py --self-test              # sampled; in ci-gates
python3 tools/agent-eval/runner.py --tier all --agent replay # the 100% gate
python3 tools/agent-eval/runner.py --tier transcribe --agent http://localhost:8088/solve --out scores.json
```

Scores carry no timestamps and every comparison is the benchmark runner's,
so a score is reproducible. `--out` writes `{agent, summary, results}` —
per task: `compiles / runs / matches / partial / failures`.

## The held-out split

`--benchmarks-dir <dir>` points the transcribe tier at any directory with
the registered case layout. Public-split numbers are contaminated for any
model trained on this repository; the honest headline number comes from a
private case set (docs/31 W2 engagements feed it) that never enters the
repo. Keep private cases in the same layout and pass the directory.

## Extend task format

`tasks/extend/<id>.toml`:

```toml
base_case = "cre/office_two_tenant"
request = "Add a refinance at year 5 at a 6.5% rate; hold everything else."
[assertions."domain.cre.debt_service"]
value = 123456.78
tolerance = 0.01
```

Assertions must be derived independently of this engine — the same
discipline as every benchmark expectation. A task whose assertion was
computed by the engine under test asserts nothing.
