---
id: results-schema
title: "Results Schema (v0.1)"
slug: "/docs/language-reference/results-schema"
source: docs/06_results_schema.md
---

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
    "monte_carlo"
  ],
  "properties": {
    "results_version": {
      "type": "string",
      "const": "0.1"
    },

    "model_hash": {
      "type": "string",
      "description": "Hash of canonical IR for traceability",
      "minLength": 8
    },

    "engine": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "version"],
      "properties": {
        "name": { "type": "string", "minLength": 1 },
        "version": { "type": "string", "minLength": 1 },
        "build": { "type": "string" }
      }
    },

    "warnings": {
      "type": "array",
      "items": { "type": "string" }
    },

    "deterministic": {
      "$ref": "#/$defs/DeterministicSection"
    },

    "monte_carlo": {
      "$ref": "#/$defs/MonteCarloSection"
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
      "required": ["amount", "currency"],
      "properties": {
        "amount": { "$ref": "#/$defs/Decimal" },
        "currency": { "$ref": "#/$defs/Currency" }
      }
    },

    "Date": {
      "type": "string",
      "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
    },

    "SeriesIndex": {
      "type": "object",
      "additionalProperties": false,
      "required": ["calendar", "start", "periods"],
      "properties": {
        "calendar": {
          "type": "string",
          "enum": ["daily", "monthly", "quarterly", "annual"]
        },
        "start": { "$ref": "#/$defs/Date" },
        "periods": { "type": "integer", "minimum": 1 }
      }
    },

    "Scalar": {
      "description": "Scalar output value (engine-computed)",
      "oneOf": [
        { "type": "number" },
        { "$ref": "#/$defs/Money" },
        { "type": "string" },
        { "type": "boolean" },
        { "type": "null" }
      ]
    },

    "Series": {
      "description": "Time series aligned to the model timeline",
      "type": "object",
      "additionalProperties": false,
      "required": ["index", "values"],
      "properties": {
        "index": { "$ref": "#/$defs/SeriesIndex" },
        "values": {
          "type": "array",
          "minItems": 1,
          "items": {
            "oneOf": [
              { "$ref": "#/$defs/Decimal" },
              { "$ref": "#/$defs/Money" },
              { "type": "null" }
            ]
          }
        }
      }
    },

    "MetricMap": {
      "type": "object",
      "description": "Named metric scalars computed by the engine per the domain pack output specification",
      "additionalProperties": { "$ref": "#/$defs/Scalar" }
    },

    "SeriesMap": {
      "type": "object",
      "description": "Named time series outputs (e.g., streams, aggregates)",
      "additionalProperties": { "$ref": "#/$defs/Series" }
    },

    "DeterministicSection": {
      "type": "object",
      "additionalProperties": false,
      "required": ["status", "metrics", "series"],
      "properties": {
        "status": {
          "type": "string",
          "enum": ["not_run", "ok", "error"]
        },
        "metrics": { "$ref": "#/$defs/MetricMap" },
        "series": { "$ref": "#/$defs/SeriesMap" },
        "errors": {
          "type": "array",
          "items": { "$ref": "#/$defs/RuntimeError" }
        }
      }
    },

    "MonteCarloSection": {
      "type": "object",
      "additionalProperties": false,
      "required": ["status", "trials", "seed", "metrics"],
      "properties": {
        "status": {
          "type": "string",
          "enum": ["not_run", "ok", "error"]
        },
        "trials": { "type": "integer", "minimum": 1 },
        "seed": { "type": "integer", "minimum": 0 },

        "metrics": {
          "type": "object",
          "description": "Named Monte Carlo metric summaries computed by the engine per the domain pack output specification",
          "additionalProperties": { "$ref": "#/$defs/MetricSummary" }
        },

        "errors": {
          "type": "array",
          "items": { "$ref": "#/$defs/RuntimeError" }
        }
      }
    },

    "MetricSummary": {
      "type": "object",
      "additionalProperties": false,
      "required": ["type", "mean", "p50"],
      "properties": {
        "type": {
          "type": "string",
          "enum": ["number", "money"]
        },

        "mean": { "$ref": "#/$defs/Scalar" },
        "stdev": { "$ref": "#/$defs/Scalar" },
        "min": { "$ref": "#/$defs/Scalar" },
        "max": { "$ref": "#/$defs/Scalar" },

        "p01": { "$ref": "#/$defs/Scalar" },
        "p05": { "$ref": "#/$defs/Scalar" },
        "p10": { "$ref": "#/$defs/Scalar" },
        "p25": { "$ref": "#/$defs/Scalar" },
        "p50": { "$ref": "#/$defs/Scalar" },
        "p75": { "$ref": "#/$defs/Scalar" },
        "p90": { "$ref": "#/$defs/Scalar" },
        "p95": { "$ref": "#/$defs/Scalar" },
        "p99": { "$ref": "#/$defs/Scalar" }
      }
    },

    "RuntimeError": {
      "type": "object",
      "additionalProperties": false,
      "required": ["code", "message"],
      "properties": {
        "code": { "type": "string", "minLength": 1 },
        "message": { "type": "string", "minLength": 1 },
        "path": { "type": "string" },
        "hint": { "type": "string" }
      }
    }
  }
}
```
