---
id: results-schema
title: "Results Schema (v0.1)"
slug: "/docs/language-reference/results-schema"
source: docs/06_results_schema.md
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
    "monte_carlo"
  ],
  "properties": {
    "results_version": {
      "type": "string",
      "const": "0.2",
      "description": "Version of this results document's shape. 0.2 added `deterministic.annual_rollup`, the optional root-level `domain_metrics`, and `Series.offset`."
    },
    "model_hash": {
      "type": "string",
      "description": "Hash of canonical IR for traceability",
      "minLength": 8
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
          "description": "One entry per period. Money for a cash series; a bare number for a dimensionless one such as a declared `state`, which has no denomination."
        },
        "offset": {
          "description": "Where in each period this series' cash falls: 0.0 at the period's open (an annuity due, or a one-shot on its date), 1.0 at its close (an ordinary annuity, the default), 0.5 for the mid-period convention. The same offset used to discount the series, and the axis `model.wal_years` and `model.payback_years` are measured on — so an ordinary annuity's first monthly collection is at 1/12 of a year, not 0. Absent on aggregates (`model.net_cash_flow`, the annual rollup), which sum streams whose placements differ. See 12_payment_timing.md. Absent on `state.` series, which are not paid and so sit nowhere in their period.",
          "type": "number"
        }
      }
    },
    "MetricMap": {
      "type": "object",
      "description": "Named metric scalars",
      "additionalProperties": {
        "$ref": "#/$defs/Scalar"
      }
    },
    "SeriesMap": {
      "type": "object",
      "description": "Named time series outputs. Keys are prefixed by what they are: `stream.<name>` and `option.<name>` are cash and carry a currency; `model.net_cash_flow` is their aggregate; `state.<name>` is a declared `state` and is NOT cash — it is a bare number with no currency and no offset, published so a recurrence can be inspected, and it never enters model.total, model.npv, the annual rollup or any domain metric.",
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
    }
  }
}
```
