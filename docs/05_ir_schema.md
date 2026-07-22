```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cfdl.dev/schemas/CFDL_v0_1_IR.schema.json",
  "title": "CFDL v0.1 Canonical IR",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "ir_version",
    "model",
    "time",
    "phases",
    "entities",
    "assumptions",
    "contracts",
    "streams",
    "events",
    "options",
    "runs",
    "required_observables",
    "required_refs",
    "provenance"
  ],
  "properties": {
    "ir_version": {
      "type": "string",
      "const": "0.1"
    },

    "model": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "currency"],
      "properties": {
        "name": { "type": "string", "minLength": 1 },
        "currency": { "$ref": "#/$defs/Currency" }
      }
    },

    "time": {
      "type": "object",
      "additionalProperties": false,
      "required": ["calendar", "start", "periods"],
      "properties": {
        "calendar": { "$ref": "#/$defs/Frequency" },
        "start": { "$ref": "#/$defs/Date" },
        "periods": { "type": "integer", "minimum": 1 }
      }
    },

    "phases": {
      "type": "array",
      "minItems": 0,
      "items": { "$ref": "#/$defs/Phase" }
    },

    "entities": {
      "type": "array",
      "minItems": 1,
      "items": { "$ref": "#/$defs/Entity" }
    },

    "assumptions": {
      "$ref": "#/$defs/Assumptions"
    },

    "curves": {
      "type": "array",
      "minItems": 0,
      "items": { "$ref": "#/$defs/Curve" }
    },

    "contracts": {
      "type": "array",
      "minItems": 0,
      "items": { "$ref": "#/$defs/Contract" }
    },

    "streams": {
      "type": "array",
      "minItems": 0,
      "items": { "$ref": "#/$defs/Stream" }
    },

    "events": {
      "type": "array",
      "minItems": 0,
      "items": { "$ref": "#/$defs/Event" }
    },

    "options": {
      "type": "array",
      "minItems": 0,
      "items": { "$ref": "#/$defs/Option" }
    },

    "runs": {
      "type": "array",
      "minItems": 1,
      "items": { "$ref": "#/$defs/Run" }
    },



    "required_observables": {
      "type": "array",
      "description": "Ontology observable IDs referenced via obs.*() calls",
      "items": { "type": "string", "minLength": 1 },
      "uniqueItems": true
    },

    "required_refs": {
      "type": "array",
      "description": "Ontology reference IDs referenced via ref('...')",
      "items": { "type": "string", "minLength": 1 },
      "uniqueItems": true
    },

    "provenance": {
      "$ref": "#/$defs/Provenance"
    }
  },

  "$defs": {
    "Id": {
      "type": "string",
      "minLength": 1,
      "maxLength": 256
    },

    "Qname": {
      "type": "string",
      "pattern": "^[A-Za-z_][A-Za-z0-9_]*(\\.[A-Za-z_][A-Za-z0-9_]*)*$"
    },

    "Date": {
      "type": "string",
      "description": "ISO date YYYY-MM-DD",
      "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
    },

    "DateRange": {
      "type": "object",
      "additionalProperties": false,
      "required": ["start", "end"],
      "properties": {
        "start": { "$ref": "#/$defs/Date" },
        "end": { "$ref": "#/$defs/Date" }
      }
    },

    "Frequency": {
      "type": "string",
      "enum": ["daily", "monthly", "quarterly", "annual"]
    },

    "Currency": {
      "type": "string",
      "description": "ISO 4217",
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

    "Rate": {
      "type": "object",
      "additionalProperties": false,
      "required": ["value"],
      "properties": {
        "value": { "$ref": "#/$defs/Decimal" },
        "basis": {
          "type": "string",
          "description": "Optional semantic basis label (e.g., 'annual')",
          "minLength": 1
        }
      }
    },

    "Expr": {
      "type": "object",
      "additionalProperties": false,
      "required": ["lang", "src"],
      "properties": {
        "lang": { "type": "string", "enum": ["cel"] },
        "src": { "type": "string", "minLength": 1 }
      }
    },

    "Phase": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "range"],
      "properties": {
        "id": { "$ref": "#/$defs/Id" },
        "range": { "$ref": "#/$defs/DateRange" }
      }
    },

    "EntityRef": {
      "type": "object",
      "additionalProperties": false,
      "required": ["symbol"],
      "properties": {
        "symbol": {
          "type": "string",
          "pattern": "^[A-Za-z_][A-Za-z0-9_]*\\.[A-Za-z_][A-Za-z0-9_]*$",
          "description": "Entity symbol like 'asset.Sunset'"
        }
      }
    },

    "Entity": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "symbol", "type", "attrs"],
      "properties": {
        "id": { "$ref": "#/$defs/Id" },
        "symbol": {
          "type": "string",
          "pattern": "^[A-Za-z_][A-Za-z0-9_]*\\.[A-Za-z_][A-Za-z0-9_]*$"
        },
        "type": { "$ref": "#/$defs/Qname" },
        "attrs": {
          "type": "object",
          "description": "Ontology-validated when pack is present; otherwise minimally typed",
          "additionalProperties": { "$ref": "#/$defs/TypedValue" }
        },
        "state": {
          "type": "object",
          "description": "Mutable runtime state fields (may be empty at compile time)",
          "additionalProperties": { "$ref": "#/$defs/TypedValue" }
        }
      }
    },

    "TypedValue": {
      "description": "Strongly-typed value union used for attrs/terms/state",
      "oneOf": [
        { "type": "string" },
        { "type": "boolean" },
        { "type": "integer" },
        { "type": "number" },
        { "$ref": "#/$defs/Date" },
        { "$ref": "#/$defs/Money" },
        { "$ref": "#/$defs/Rate" },
        { "$ref": "#/$defs/Expr" },
        {
          "type": "array",
          "items": { "$ref": "#/$defs/TypedValue" }
        },
        {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/TypedValue" }
        }
      ]
    },

    "Assumptions": {
      "type": "object",
      "additionalProperties": false,
      "required": ["constants", "random"],
      "properties": {
        "constants": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/AssumeConstant" }
        },
        "random": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/AssumeRandom" }
        }
      }
    },

    "AssumeConstant": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "expr", "type"],
      "properties": {
        "name": { "$ref": "#/$defs/Id" },
        "expr": { "$ref": "#/$defs/Expr" },
        "type": { "$ref": "#/$defs/ValueTypeId" }
      }
    },

    "AssumeRandom": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "dist", "type"],
      "properties": {
        "name": { "$ref": "#/$defs/Id" },
        "dist": { "$ref": "#/$defs/Distribution" },
        "type": { "$ref": "#/$defs/ValueTypeId" }
      }
    },

    "ValueTypeId": {
      "type": "string",
      "enum": [
        "String",
        "Bool",
        "Int",
        "Decimal",
        "Date",
        "Money",
        "Rate"
      ]
    },

    "Distribution": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind", "params"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": ["Normal", "LogNormal", "Uniform", "Triangular"]
        },
        "params": {
          "type": "object",
          "additionalProperties": { "type": ["number", "string", "boolean"] }
        },
        "clip": {
          "type": "array",
          "items": { "type": "number" },
          "minItems": 2,
          "maxItems": 2
        }
      }
    },

    "Contract": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "type",
        "subject",
        "term",
        "currency",
        "terms",
        "effects",
        "provenance"
      ],
      "properties": {
        "id": { "$ref": "#/$defs/Id" },
        "name": { "$ref": "#/$defs/Id" },
        "type": { "$ref": "#/$defs/Qname" },
        "subject": { "$ref": "#/$defs/EntityRef" },
        "term": { "$ref": "#/$defs/DateRange" },
        "currency": { "$ref": "#/$defs/Currency" },
        "parties": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/TypedValue" }
        },
        "tags": {
          "type": "object",
          "additionalProperties": { "$ref": "#/$defs/TypedValue" }
        },
        "terms": {
          "type": "object",
          "description": "Contract terms; pack may validate and type-check",
          "additionalProperties": { "$ref": "#/$defs/TypedValue" }
        },
        "effects": {
          "$ref": "#/$defs/Effects"
        },
        "provenance": { "$ref": "#/$defs/NodeProvenance" }
      }
    },

    "Effects": {
      "type": "object",
      "additionalProperties": false,
      "required": ["streams"],
      "properties": {
        "streams": {
          "type": "array",
          "items": { "$ref": "#/$defs/Stream" }
        }
      }
    },

    "Stream": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "owner",
        "direction",
        "currency",
        "schedule",
        "amount",
        "active_when",
        "provenance"
      ],
      "properties": {
        "id": { "$ref": "#/$defs/Id" },
        "name": { "$ref": "#/$defs/Id" },
        "owner": { "$ref": "#/$defs/EntityRef" },
        "direction": { "$ref": "#/$defs/Direction" },
        "currency": { "$ref": "#/$defs/Currency" },
        "schedule": { "$ref": "#/$defs/Schedule" },
        "amount": { "$ref": "#/$defs/Expr" },
        "active_when": {
          "description": "If omitted in source, compiler should emit true",
          "$ref": "#/$defs/Expr"
        },
        "provenance": { "$ref": "#/$defs/NodeProvenance" }
      }
    },

    "Direction": {
      "type": "string",
      "enum": ["inflow", "outflow"]
    },

    "Schedule": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "OnDate",
            "Every",
            "PhaseEnter",
            "EveryPhase"
          ]
        },

        "on": { "$ref": "#/$defs/Date" },

        "every": { "$ref": "#/$defs/Frequency" },
        "from": { "$ref": "#/$defs/Date" },
        "to": { "$ref": "#/$defs/Date" },

        "on_rule": {
          "description": "Optional rule for day-of-month or weekday sets",
          "$ref": "#/$defs/OnRule"
        },

        "convention": { "$ref": "#/$defs/BusinessDayConvention" },
        "calendar": {
          "type": "string",
          "description": "Named business day calendar (e.g., 'NYSE')"
        },
        "stub": { "$ref": "#/$defs/StubPolicy" },

        "except": {
          "type": "array",
          "items": { "$ref": "#/$defs/Date" }
        },
        "also": {
          "type": "array",
          "items": { "$ref": "#/$defs/Date" }
        },

        "phase": {
          "type": "string",
          "description": "Phase name for PhaseEnter/EveryPhase"
        }
      },
      "allOf": [
        {
          "if": { "properties": { "kind": { "const": "OnDate" } } },
          "then": { "required": ["on"] }
        },
        {
          "if": { "properties": { "kind": { "const": "Every" } } },
          "then": { "required": ["every", "from", "to"] }
        },
        {
          "if": { "properties": { "kind": { "const": "PhaseEnter" } } },
          "then": { "required": ["phase"] }
        },
        {
          "if": { "properties": { "kind": { "const": "EveryPhase" } } },
          "then": { "required": ["every", "phase"] }
        }
      ]
    },

    "OnRule": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "DayOfMonth",
            "EndOfMonth",
            "Weekdays"
          ]
        },
        "day": { "type": "integer", "minimum": 1, "maximum": 31 },
        "weekdays": {
          "type": "array",
          "items": { "$ref": "#/$defs/Weekday" },
          "minItems": 1,
          "uniqueItems": true
        }
      },
      "allOf": [
        {
          "if": { "properties": { "kind": { "const": "DayOfMonth" } } },
          "then": { "required": ["day"] }
        },
        {
          "if": { "properties": { "kind": { "const": "Weekdays" } } },
          "then": { "required": ["weekdays"] }
        }
      ]
    },

    "Weekday": {
      "type": "string",
      "enum": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    },

    "BusinessDayConvention": {
      "type": "string",
      "enum": [
        "none",
        "following",
        "modified_following",
        "preceding",
        "modified_preceding"
      ]
    },

    "StubPolicy": {
      "type": "string",
      "enum": [
        "none",
        "short_front",
        "short_back",
        "long_front",
        "long_back"
      ]
    },

    "Event": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "name", "when", "actions", "provenance"],
      "properties": {
        "id": { "$ref": "#/$defs/Id" },
        "name": { "$ref": "#/$defs/Id" },
        "when": { "$ref": "#/$defs/Expr" },
        "actions": {
          "type": "array",
          "items": { "$ref": "#/$defs/Action" }
        },
        "provenance": { "$ref": "#/$defs/NodeProvenance" }
      }
    },

    "Action": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "SetEntityField",
            "ActivateStream",
            "DeactivateStream",
            "ActivateContract",
            "DeactivateContract",
            "ExerciseOption"
          ]
        },

        "entity": { "$ref": "#/$defs/EntityRef" },
        "field": { "$ref": "#/$defs/Id" },
        "value": { "$ref": "#/$defs/TypedValue" },

        "stream": { "$ref": "#/$defs/Id" },
        "contract": { "$ref": "#/$defs/Id" },
        "option": { "$ref": "#/$defs/Id" }
      },
      "allOf": [
        {
          "if": { "properties": { "kind": { "const": "SetEntityField" } } },
          "then": { "required": ["entity", "field", "value"] }
        },
        {
          "if": { "properties": { "kind": { "const": "ActivateStream" } } },
          "then": { "required": ["stream"] }
        },
        {
          "if": { "properties": { "kind": { "const": "DeactivateStream" } } },
          "then": { "required": ["stream"] }
        },
        {
          "if": { "properties": { "kind": { "const": "ActivateContract" } } },
          "then": { "required": ["contract"] }
        },
        {
          "if": { "properties": { "kind": { "const": "DeactivateContract" } } },
          "then": { "required": ["contract"] }
        },
        {
          "if": { "properties": { "kind": { "const": "ExerciseOption" } } },
          "then": { "required": ["option"] }
        }
      ]
    },

    "Option": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "name", "type", "exercise_when", "payoff", "provenance"],
      "properties": {
        "id": { "$ref": "#/$defs/Id" },
        "name": { "$ref": "#/$defs/Id" },
        "type": { "$ref": "#/$defs/Qname" },
        "exercisable_in_phase": { "$ref": "#/$defs/Id" },
        "exercise_when": { "$ref": "#/$defs/Expr" },
        "payoff": { "$ref": "#/$defs/Expr" },
        "provenance": { "$ref": "#/$defs/NodeProvenance" }
      }
    },

    "Run": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": {
          "type": "string",
          "enum": ["deterministic", "monte_carlo"]
        },
        "trials": { "type": "integer", "minimum": 1 },
        "seed": { "type": "integer", "minimum": 0 }
      },
      "allOf": [
        {
          "if": { "properties": { "kind": { "const": "monte_carlo" } } },
          "then": { "required": ["trials", "seed"] }
        }
      ]
    },



    "Provenance": {
      "type": "object",
      "additionalProperties": false,
      "required": ["sources", "compiler"],
      "properties": {
        "sources": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 },
          "minItems": 1
        },
        "compiler": {
          "type": "object",
          "additionalProperties": false,
          "required": ["name", "version", "hash"],
          "properties": {
            "name": { "type": "string", "minLength": 1 },
            "version": { "type": "string", "minLength": 1 },
            "hash": { "type": "string", "minLength": 8 },
            "notes": {
              "type": "array",
              "items": { "type": "string", "minLength": 1 }
            }
          }
        }
      }
    },

    "NodeProvenance": {
      "type": "object",
      "additionalProperties": false,
      "required": ["source_file", "source_span"],
      "properties": {
        "source_file": { "type": "string", "minLength": 1 },
        "source_span": {
          "type": "object",
          "additionalProperties": false,
          "required": ["start_line", "start_col", "end_line", "end_col"],
          "properties": {
            "start_line": { "type": "integer", "minimum": 1 },
            "start_col": { "type": "integer", "minimum": 1 },
            "end_line": { "type": "integer", "minimum": 1 },
            "end_col": { "type": "integer", "minimum": 1 }
          }
        },
        "notes": { "type": "string" },
        "generated_by": { "$ref": "#/$defs/GeneratedBy" }
      }
    },

    "GeneratedBy": {
      "type": "object",
      "additionalProperties": false,
      "required": ["pack", "rule_id"],
      "properties": {
        "pack": { "$ref": "#/$defs/PackRef" },
        "rule_id": { "type": "string", "minLength": 1 }
      }
    },

    "PackRef": {
      "type": "object",
      "additionalProperties": false,
      "required": ["name", "version"],
      "properties": {
        "name": { "type": "string", "minLength": 1 },
        "version": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

Schema sync guardrail:
- `docs/CFDL_v0_1_IR.schema.json` is the canonical machine-readable artifact.
- The fenced JSON block in this markdown file must remain semantically identical.
- Before merging schema changes, run a parity check that parses both and compares
  JSON object equality to prevent drift.

