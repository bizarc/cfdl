---
id: results-schema
title: "Results schema (v0.1)"
slug: "/docs/specification/results-schema"
description: "The JSON schema for a results document: series, metrics, statements, and the provenance of the run that produced them."
source: docs/06_results_schema.md
generated: full
layer: specification
---

<!-- GENERATED from docs/schemas/results.schema.json — do not edit by hand.
     tools/check-results-schema.py fails the build if this drifts.
     This page existed as an independently maintained copy and was four
     releases behind: it declared results_version 0.1 while the engine
     emitted 0.2, and omitted two whole sections. -->

# Results schema

The shape of a `cfdl run` results document. This is the published contract,
also served at `cfdl.dev/schemas`; every committed results golden is validated
against it by `make results-schema`.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cfdl.dev/schemas/CFDL_v0_1_Results.schema.json",
  "title": "CFDL v0.1 Results",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "results_version",
    "model_hash",
    "engine",
    "warnings",
    "deterministic",
    "scenarios",
    "monte_carlo",
    "ledger_hash"
  ],
  "properties": {
    "results_version": {
      "type": "string",
      "const": "0.8",
      "description": "Schema version of this document. 0.8 adds `slices` — declared partial selections with their matched streams, net series and figures, and no reconciliation block by design. 0.7 publishes the model's entity graph (`graph`) and attributes each stream series to its owning entity and category. 0.6 nests an act's own acts under it as `children`. 0.5 added the machine's `transition` journal action. 0.4 added the account journal actions. 0.3 added `ledger_hash`, the optional `inputs` section, and `category` on IR streams."
    },
    "model_hash": {
      "type": "string",
      "description": "Hash of canonical IR for traceability",
      "minLength": 8
    },
    "ledger_hash": {
      "type": "string",
      "description": "SHA-256 over the canonical form of the deterministic ledger — `deterministic.series` and `deterministic.annual_rollup`. Together with `model_hash` and `engine` this closes the chain: identical inputs on an identical engine must reproduce an identical ledger_hash. It covers the LEDGER, not the metrics: NPV and IRR are derived FROM the ledger, so including them would make the hash move for a reason the ledger did not. It is therefore invariant to the discount rate, which is correct — the ledger is cash before discounting."
    },
    "engine": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "version"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "version": {
          "type": "string",
          "minLength": 1
        },
        "build": {
          "type": "string"
        }
      }
    },
    "warnings": {
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "inputs": {
      "$ref": "#/$defs/InputsSection"
    },
    "deterministic": {
      "$ref": "#/$defs/DeterministicSection"
    },
    "scenarios": {
      "$ref": "#/$defs/ScenariosSection"
    },
    "monte_carlo": {
      "$ref": "#/$defs/MonteCarloSection"
    },
    "domain_metrics": {
      "$ref": "#/$defs/DomainMetrics"
    },
    "statements": {
      "$ref": "#/$defs/StatementsSection"
    },
    "graph": {
      "$ref": "#/$defs/ResultsGraph"
    },
    "slices": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/SliceResult"
      }
    }
  },
  "$defs": {
    "Currency": {
      "type": "string",
      "pattern": "^[A-Z]{3}$"
    },
    "Decimal": {
      "type": "number"
    },
    "Money": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "amount",
        "currency"
      ],
      "properties": {
        "amount": {
          "$ref": "#/$defs/Decimal"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        }
      }
    },
    "Date": {
      "type": "string",
      "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
    },
    "SeriesIndex": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "calendar",
        "start",
        "periods"
      ],
      "properties": {
        "calendar": {
          "type": "string",
          "enum": [
            "daily",
            "monthly",
            "quarterly",
            "annual"
          ]
        },
        "start": {
          "$ref": "#/$defs/Date"
        },
        "periods": {
          "type": "integer",
          "minimum": 1
        }
      }
    },
    "Scalar": {
      "description": "Scalar metric output",
      "oneOf": [
        {
          "type": "number"
        },
        {
          "$ref": "#/$defs/Money"
        },
        {
          "type": "string"
        },
        {
          "type": "boolean"
        },
        {
          "type": "null"
        }
      ]
    },
    "Series": {
      "description": "Time series aligned to the model timeline",
      "type": "object",
      "additionalProperties": false,
      "required": [
        "index",
        "values"
      ],
      "properties": {
        "index": {
          "$ref": "#/$defs/SeriesIndex"
        },
        "values": {
          "type": "array",
          "minItems": 1,
          "items": {
            "oneOf": [
              {
                "$ref": "#/$defs/Decimal"
              },
              {
                "$ref": "#/$defs/Money"
              },
              {
                "type": "null"
              }
            ]
          },
          "description": "One entry per period. Money for a cash series; a bare number for a dimensionless one such as an entity field, which has no denomination."
        },
        "offset": {
          "description": "Where in each period this series' cash falls: 0.0 at the period's open (an annuity due, or a one-shot on its date), 1.0 at its close (an ordinary annuity, the default), 0.5 for the mid-period convention. The same offset used to discount the series, and the axis `model.wal_years` and `model.payback_years` are measured on — so an ordinary annuity's first monthly collection is at 1/12 of a year, not 0. Absent on aggregates (`model.net_cash_flow`, the annual rollup), which sum streams whose placements differ. See 12_payment_timing.md. Absent on field series, which are not paid and so sit nowhere in their period.",
          "type": "number"
        },
        "entity": {
          "type": "string",
          "description": "The entity this stream is attached to (`asset.tower`, `container.fund`). Present on stream series only — a subtotal spans owners and an aggregate has none. With `graph`, this is what lets a consumer attribute cash to a thing without the IR: name inspection is not a substitute, because a pack-lowered stream's name does not contain its owner's symbol."
        },
        "category": {
          "type": "string",
          "description": "The stream's declared category (`operating.revenue.base_rent`). Present on categorized stream series only. Ownership says whose cash; category says what kind — the two axes a selection needs."
        }
      }
    },
    "MetricMap": {
      "type": "object",
      "description": "Named metric scalars. The prefix says who minted the number: `model.*` is the engine's (total, npv, irr, moic, payback, wal), `domain.<pack>.*` is the active pack's, and `metric.<name>` is one the MODEL declared (`docs/01` §15.3) — a figure this deal solved for, evaluated once at the horizon over the finished projection. `stream.<name>.total` is a stream's own sum. A declared metric appears in every scenario summary as well, since scenarios and the deterministic block publish the same map.",
      "additionalProperties": {
        "$ref": "#/$defs/Scalar"
      }
    },
    "SeriesMap": {
      "type": "object",
      "description": "Named time series outputs. Keys are prefixed by what they are: `stream.<name>` and `option.<name>` are cash and carry a currency; `model.net_cash_flow` is their aggregate; `<family>.<entity>.<field>` is an entity field and is NOT cash — it is a bare number with no currency and no offset, published so a recurrence can be inspected, and it never enters model.total, model.npv, the annual rollup or any domain metric. `entity.<symbol>.net_cash_flow` is an entity's cash AGGREGATED BY RELATION — its own streams plus every descendant's, following `part_of` rather than a name prefix, so a building's cash is its units' cash because they ARE its units. An entity with no children carries its own streams only, which is the pool that models collective behavior directly; the grain is the modeller's choice. Like a subtotal it is a fold OF the cash and never counts AS cash — excluded from model.total, model.npv, model.net_cash_flow and the annual rollup, because counting a parent and its children would double what it touches. `domain.<pack>.<name>` is a per-period SUBTOTAL — a declared aggregation of the classified streams. Money for a sum, a bare number or `null` for a ratio whose denominator vanishes. Like a field, it never enters model.total, model.npv, model.net_cash_flow or the per-stream annual rollup: it is an aggregation OF the cash, so counting it as cash would double what it touches. It carries no `offset`, because a subtotal spans streams that may settle at different points in a period and so has no single placement to claim.",
      "additionalProperties": {
        "$ref": "#/$defs/Series"
      }
    },
    "DeterministicSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "status",
        "metrics",
        "series"
      ],
      "properties": {
        "status": {
          "type": "string",
          "enum": [
            "not_run",
            "ok",
            "error"
          ]
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        },
        "series": {
          "$ref": "#/$defs/SeriesMap"
        },
        "errors": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/RuntimeError"
          }
        },
        "annual_rollup": {
          "$ref": "#/$defs/AnnualRollupSection"
        },
        "transitions": {
          "type": "array",
          "description": "Every state change an event made, in the order it happened — the audit trail for whether and when something occurred. Entity state is otherwise unobservable: nothing else distinguishes an event that fired against a misspelled target from an event that never fired, and without this a case cannot assert a transition. Recorded even when the value does not change, because the question the log answers is whether the event fired. Omitted when a model has no events. Visibility is two rules, not one: an event or option guard reads the state as the period OPENED, so declaration order cannot change an answer; a stream reads it as the period CLOSED, so a transition takes effect in the period it fires.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "period",
              "date",
              "entity",
              "field",
              "to",
              "event"
            ],
            "properties": {
              "period": {
                "type": "integer",
                "minimum": 0
              },
              "date": {
                "type": "string"
              },
              "entity": {
                "type": "string"
              },
              "field": {
                "type": "string"
              },
              "from": {
                "type": "string",
                "description": "The value before. Absent when the field had none — which, for a typed entity with a lifecycle, should not happen, because it opens in its declared initial state."
              },
              "to": {
                "type": "string"
              },
              "event": {
                "type": "string",
                "description": "The event that fired. A transition always has a cause."
              }
            }
          }
        },
        "journal": {
          "type": "array",
          "description": "Every causal act the run performed, with what became of it, in the order the engine performed them. `transitions` records field CHANGES; the journal answers the question a reviewer asks — what did the model DO, and did each thing it was asked to do happen. An action that was declined, ignored or overridden changes nothing and so appears nowhere else: an `activate stream` that lost to the stream's own `active when` used to leave no trace at all. One row per act, and one row TYPE — an act whose effects are its own acts nests them as `children` rather than flattening them beside itself, which is what a transition and its arrival actions are (0.6). Omitted when a model has no events, options or waterfalls, so such a model publishes exactly what it published before.",
          "items": {
            "$ref": "#/$defs/JournalEntry"
          }
        }
      }
    },
    "ScenariosSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "status",
        "summaries"
      ],
      "properties": {
        "status": {
          "type": "string",
          "enum": [
            "not_run",
            "ok",
            "error"
          ]
        },
        "summaries": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/ScenarioSummary"
          }
        },
        "errors": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/RuntimeError"
          }
        }
      }
    },
    "ScenarioSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "metrics"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        }
      }
    },
    "MonteCarloSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "status",
        "trials",
        "seed",
        "metrics",
        "trial_summaries"
      ],
      "properties": {
        "status": {
          "type": "string",
          "enum": [
            "not_run",
            "ok",
            "error"
          ]
        },
        "trials": {
          "type": "integer",
          "minimum": 1
        },
        "seed": {
          "type": "integer",
          "minimum": 0
        },
        "metrics": {
          "type": "object",
          "description": "Named Monte Carlo metric summaries",
          "additionalProperties": {
            "$ref": "#/$defs/MetricSummary"
          }
        },
        "trial_summaries": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/TrialSummary"
          }
        },
        "aggregates": {
          "$ref": "#/$defs/MonteCarloAggregates"
        },
        "errors": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/RuntimeError"
          }
        },
        "journal": {
          "type": "array",
          "description": "When each act happened across the trials, and how often — the question a stochastic run asks of the journal. A per-trial log is the wrong shape: trials x acts of output, and nobody reads ten thousand copies of the same sequence. So each distinct act gets one row, bounded by the model rather than the trial count, carrying the share of trials in which it occurred and the distribution over the period it FIRST did. Omitted when no trial recorded any act.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "actor",
              "action",
              "target",
              "outcome",
              "trials_occurred",
              "share",
              "first_period"
            ],
            "properties": {
              "actor": {
                "type": "string",
                "description": "The act's identity, matching the deterministic journal's own fields so a summary lines up against a single run's trail."
              },
              "action": {
                "type": "string"
              },
              "target": {
                "type": "string"
              },
              "outcome": {
                "type": "string"
              },
              "trials_occurred": {
                "type": "integer",
                "minimum": 0,
                "description": "Trials in which this act occurred at least once."
              },
              "share": {
                "type": "number",
                "minimum": 0,
                "maximum": 1
              },
              "first_period": {
                "type": "object",
                "additionalProperties": false,
                "required": [
                  "min",
                  "p10",
                  "median",
                  "p90",
                  "max",
                  "mean"
                ],
                "description": "Over the trials where the act occurred, the period it first did. Quantiles are nearest-rank order statistics rather than interpolated, because a quantile of periods should be a period: \"the covenant first broke around month 9\", not month 9.5. The mean stays fractional, being explicitly an average rather than an observation.",
                "properties": {
                  "min": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "p10": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "median": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "p90": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "max": {
                    "type": "integer",
                    "minimum": 0
                  },
                  "mean": {
                    "type": "number",
                    "minimum": 0
                  }
                }
              }
            }
          }
        }
      }
    },
    "TrialSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "trial",
        "metrics"
      ],
      "properties": {
        "trial": {
          "type": "integer",
          "minimum": 0
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        }
      }
    },
    "MonteCarloAggregates": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "npv"
      ],
      "properties": {
        "npv": {
          "$ref": "#/$defs/NpvAggregate"
        }
      }
    },
    "NpvAggregate": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "mean",
        "median",
        "stddev",
        "p_negative"
      ],
      "properties": {
        "mean": {
          "$ref": "#/$defs/Decimal"
        },
        "median": {
          "$ref": "#/$defs/Decimal"
        },
        "stddev": {
          "$ref": "#/$defs/Decimal"
        },
        "p_negative": {
          "$ref": "#/$defs/Decimal"
        }
      }
    },
    "MetricSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "type",
        "mean",
        "p50"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "number",
            "money"
          ]
        },
        "mean": {
          "$ref": "#/$defs/Scalar"
        },
        "stdev": {
          "$ref": "#/$defs/Scalar"
        },
        "min": {
          "$ref": "#/$defs/Scalar"
        },
        "max": {
          "$ref": "#/$defs/Scalar"
        },
        "p01": {
          "$ref": "#/$defs/Scalar"
        },
        "p05": {
          "$ref": "#/$defs/Scalar"
        },
        "p10": {
          "$ref": "#/$defs/Scalar"
        },
        "p25": {
          "$ref": "#/$defs/Scalar"
        },
        "p50": {
          "$ref": "#/$defs/Scalar"
        },
        "p75": {
          "$ref": "#/$defs/Scalar"
        },
        "p90": {
          "$ref": "#/$defs/Scalar"
        },
        "p95": {
          "$ref": "#/$defs/Scalar"
        },
        "p99": {
          "$ref": "#/$defs/Scalar"
        }
      }
    },
    "RuntimeError": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "code",
        "message"
      ],
      "properties": {
        "code": {
          "type": "string",
          "minLength": 1
        },
        "message": {
          "type": "string",
          "minLength": 1
        },
        "path": {
          "type": "string"
        },
        "hint": {
          "type": "string"
        }
      }
    },
    "MetricLineage": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "numerator_streams",
        "denominator_streams",
        "formula"
      ],
      "description": "Where a domain metric's value came from: the stream selectors it summed and the human-readable formula the pack declared. Emitted so a metric can be audited without reading the pack.",
      "properties": {
        "numerator_streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "denominator_streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "formula": {
          "type": "string"
        }
      }
    },
    "DomainMetrics": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "pack",
        "metrics",
        "lineage"
      ],
      "description": "Pack-defined metrics, present only when the run named a pack (`--pack <name>`). Engine-universal metrics live in `deterministic.metrics`; these are the domain's own, declared in the pack's metrics.toml.",
      "properties": {
        "pack": {
          "type": "string"
        },
        "metrics": {
          "$ref": "#/$defs/MetricMap"
        },
        "lineage": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/MetricLineage"
          }
        }
      }
    },
    "AnnualRollupSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "series"
      ],
      "description": "The deterministic series aggregated to annual buckets, for reporting a sub-annual model on a yearly grid. Present whenever the deterministic run succeeded. Carries no `offset`: an annual bucket sums periods whose placements differ.",
      "properties": {
        "series": {
          "$ref": "#/$defs/SeriesMap"
        }
      }
    },
    "InputsSection": {
      "type": "object",
      "additionalProperties": false,
      "description": "What went in, above the line items — the top of the audit chain. Absent when the model declares neither assumptions nor pack-lowered streams.",
      "properties": {
        "resolved": {
          "type": "object",
          "additionalProperties": {
            "type": "number"
          },
          "description": "Evaluated `assume` values, as `inputs.<name>` resolves them. In a deterministic run a random assumption resolves to its clipped CENTRAL value rather than to a draw; publishing it here is what stops that being invisible."
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "object"
          },
          "description": "Per-stream record of the contract terms a pack rule consumed to strike it, passed through from the IR's `stream_inputs` verbatim. See the IR schema's StreamInputs. Hand-written streams have no entry, because no rule struck them."
        },
        "quantiles": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/QuantileCall"
          },
          "description": "Which slice of a declared quantile each expression asked for, and what it resolved to. Passed through from the IR's quantile_inputs verbatim. A nonlinear input whose evaluation is not published is a number no reviewer can check: the top 2% of hours averaging 340.00 is the fact that explains the revenue, and the declaration alone does not state it."
        }
      }
    },
    "StatementsSection": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "pack",
        "statements"
      ],
      "description": "Statements the active pack declares, rendered against this run. Rows carry order, labels, depth and a display sign; they compute nothing the engine has not already aggregated. Absent when the pack declares no statement.",
      "properties": {
        "pack": {
          "type": "string"
        },
        "statements": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Statement"
          }
        }
      }
    },
    "Statement": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "label",
        "default",
        "grain",
        "rows",
        "reconciliation"
      ],
      "properties": {
        "id": {
          "type": "string"
        },
        "label": {
          "type": "string"
        },
        "default": {
          "type": "boolean"
        },
        "grain": {
          "$ref": "#/$defs/StatementGrain"
        },
        "rows": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/StatementRow"
          }
        },
        "reconciliation": {
          "$ref": "#/$defs/StatementReconciliation"
        },
        "diagnostics": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/StatementDiagnostic"
          },
          "description": "Completeness findings. Empty is the healthy case."
        }
      }
    },
    "StatementGrain": {
      "type": "object",
      "additionalProperties": false,
      "description": "The grain this statement reports at, and one ready-to-render label per column. Published because a consumer cannot derive it: an annual statement over a monthly model has ten values where the model has 120, and nothing else in the document says which ten periods those are.",
      "required": [
        "calendar",
        "start",
        "labels"
      ],
      "properties": {
        "calendar": {
          "type": "string",
          "description": "monthly | quarterly | annual | daily — the bucketing, not the model grid."
        },
        "start": {
          "type": "string"
        },
        "labels": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "One per column, aligned with every row's `values`."
        }
      }
    },
    "StatementRow": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind",
        "depth",
        "display_sign"
      ],
      "description": "One row. `residual` is emitted for cash no row claimed and cannot be authored; a `spacer` carries no values.",
      "properties": {
        "kind": {
          "enum": [
            "line",
            "subtotal",
            "ratio",
            "spacer",
            "residual"
          ]
        },
        "label": {
          "type": "string"
        },
        "depth": {
          "type": "integer",
          "minimum": 0
        },
        "display_sign": {
          "type": "number",
          "enum": [
            1,
            -1
          ],
          "description": "How to RENDER the sign. `values` is always the signed arithmetic quantity, so a consumer that ignores this still adds up correctly. -1 is how a deduction prints as a positive number in a 'less:' row while still being counted negatively — a line can be shown AND counted."
        },
        "values": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/SeriesValue"
          }
        },
        "total": {
          "type": "number",
          "description": "Lifetime total. Absent for a ratio, where summing a column of ratios answers nothing, and for a spacer."
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "The streams this row drew from — what makes a published figure traceable without a flow ledger."
        }
      }
    },
    "StatementReconciliation": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "bottom_line",
        "model_total",
        "residual"
      ],
      "description": "Does the statement account for the model's cash? Published always and asserted rather than corrected: a bottom line that quietly differs from model.total is the failure this exists to make visible.",
      "properties": {
        "bottom_line": {
          "type": "number"
        },
        "model_total": {
          "type": "number"
        },
        "residual": {
          "type": "number"
        }
      }
    },
    "StatementDiagnostic": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "code",
        "message"
      ],
      "properties": {
        "code": {
          "type": "string"
        },
        "message": {
          "type": "string"
        }
      }
    },
    "SeriesValue": {
      "description": "A series point: money, a bare number, or null where undefined."
    },
    "QuantileCall": {
      "type": "object",
      "required": [
        "quantile",
        "function",
        "args"
      ],
      "additionalProperties": false,
      "properties": {
        "quantile": {
          "type": "string",
          "description": "The quantile named at the call site."
        },
        "function": {
          "type": "string",
          "enum": [
            "quantile_at",
            "quantile_mean",
            "quantile_of"
          ]
        },
        "args": {
          "type": "array",
          "items": {
            "type": "number"
          },
          "description": "The literal arguments after the name, in source order. Empty when they were not literals."
        },
        "value": {
          "type": "number",
          "description": "What the call resolves to, rounded to the engine's published-number policy so it agrees exactly with the ledger figure it explains. ABSENT when an argument is not a literal — the call is still listed, because a silently omitted call site would read as a model that never made one."
        }
      }
    },
    "JournalEntry": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "period",
        "date",
        "actor",
        "action",
        "target",
        "outcome"
      ],
      "properties": {
        "period": {
          "type": "integer",
          "minimum": 0
        },
        "date": {
          "type": "string"
        },
        "actor": {
          "type": "string",
          "description": "Who acted, qualified by kind: `event:<name>`, `waterfall:<name>`, `option:<name>`, `stream:<name>`. Qualified because a waterfall and an event may share a name and the log must not conflate them."
        },
        "action": {
          "type": "string",
          "enum": [
            "set",
            "activate_stream",
            "deactivate_stream",
            "activate_contract",
            "deactivate_contract",
            "exercise_option",
            "pay",
            "inflow",
            "allocate_in",
            "allocate_out",
            "transition"
          ]
        },
        "target": {
          "type": "string",
          "description": "What was acted on — a field path, a stream name, or a step and its payee."
        },
        "outcome": {
          "type": "string",
          "enum": [
            "applied",
            "declined",
            "overridden",
            "ignored",
            "failed"
          ],
          "description": "`applied` is the only one that changed anything. `declined` was refused for a stated reason. `overridden` was done and then lost to a stronger declaration — a stream activation against a false `active when`, or a waterfall step against a short pot. `ignored` is an action the engine does not execute yet. `failed` means the action's own expression did not evaluate."
        },
        "from": {
          "type": "string"
        },
        "to": {
          "type": "string"
        },
        "amount": {
          "type": "number",
          "description": "What the step allocated. Allocated, not transferred: a waterfall is an ordered allocation over a pot, deciding what each step is entitled to out of what remains. Whether that cash physically settles is a question the language does not model."
        },
        "pot_before": {
          "type": "number",
          "description": "The pot before the step drew on it, so a short pot is visible as the reason a step was allocated less than it was owed."
        },
        "pot_after": {
          "type": "number"
        },
        "note": {
          "type": "string",
          "description": "Why, when the outcome is not `applied`."
        },
        "children": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/JournalEntry"
          },
          "description": "What this occurrence DID, when the occurrence and its effects are two different things. A transition's arrival actions are its children rather than its siblings because the tie between them is real: sharing a period and an entity only implies it, and a reader reconstructing which `set` belonged to which arrival would be guessing where several entities move in one period. Absent where an act has no composite effects, which is most of them."
        }
      },
      "description": "One act, and what became of it. An act that DID something composite carries what it did as `children` — a transition is the occurrence, its arrival actions are its effects."
    },
    "GraphEntity": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "symbol",
        "family"
      ],
      "properties": {
        "symbol": {
          "type": "string",
          "description": "The reference the model uses everywhere — `asset.tower`."
        },
        "family": {
          "type": "string",
          "description": "The symbol's first segment: asset, party, or container."
        },
        "type": {
          "type": "string",
          "description": "The ontology type the declaration states, when it states one."
        },
        "id": {
          "type": "string",
          "description": "The stable identity the model carries for a layer above it — the literal field `id`, engine-opaque, unique within the model (E1360)."
        },
        "parent": {
          "type": "string",
          "description": "The `part of` parent, when the model groups this entity."
        }
      }
    },
    "ResultsGraph": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "entities"
      ],
      "description": "The model's entity graph, published so a consumer holding results alone can build the hierarchy view — who is part of what, what each thing is, and the stable identity a governance layer assigned. Values, not vocabulary: the pack's type roster lives in the pack; this is the graph THIS model declared.",
      "properties": {
        "entities": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/GraphEntity"
          }
        }
      }
    },
    "SliceSelection": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "entities": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "types": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "categories": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "except_streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "except_categories": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "except_entities": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      }
    },
    "SliceResult": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "selection",
        "streams",
        "net",
        "metrics"
      ],
      "description": "A declared slice and what it came to (docs/01 §15.4): the selection as lineage, every stream it matched (empty is published, not omitted), the net per-period series, and total/npv/irr over the matched streams on the model's own axis. NO reconciliation block, by design — a slice is partial, and must be seen to be.",
      "properties": {
        "id": {
          "type": "string"
        },
        "selection": {
          "$ref": "#/$defs/SliceSelection"
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "net": {
          "$ref": "#/$defs/Series"
        },
        "metrics": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/Scalar"
          }
        }
      }
    }
  }
}
```
