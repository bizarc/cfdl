---
id: ir-schema
title: "IR schema (v0.1)"
slug: "/docs/specification/ir-schema"
source: docs/05_ir_schema.md
generated: full
layer: specification
---

<!-- GENERATED from docs/schemas/ir.schema.json — do not edit by hand.
     tools/check-ir-schema.py fails the build if this drifts. This page
     was an independently maintained copy, which is how the results
     schema drifted four releases before anyone noticed. -->

# IR schema

The shape of a `cfdl compile` IR document. This is the published contract,
also served at `cfdl.dev/schemas`; every committed IR golden is validated
against it by `make ir-schema`.

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
      "required": [
        "name",
        "currency"
      ],
      "properties": {
        "name": {
          "type": "string",
          "minLength": 1
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        }
      }
    },
    "time": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "calendar",
        "start",
        "periods"
      ],
      "properties": {
        "calendar": {
          "$ref": "#/$defs/Frequency"
        },
        "start": {
          "$ref": "#/$defs/Date"
        },
        "periods": {
          "type": "integer",
          "minimum": 1
        },
        "projection": {
          "type": "integer",
          "minimum": 0
        }
      }
    },
    "phases": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Phase"
      }
    },
    "entities": {
      "type": "array",
      "minItems": 1,
      "items": {
        "$ref": "#/$defs/Entity"
      }
    },
    "assumptions": {
      "$ref": "#/$defs/Assumptions"
    },
    "curves": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Curve"
      }
    },
    "waterfalls": {
      "type": "array",
      "minItems": 0,
      "description": "Ordered allocations of a pot — a priority of payments. Steps run in declaration order after the period's fields and streams are known; each takes min(max(0, its amount), what remains). Omitted when a model declares none.",
      "items": {
        "$ref": "#/$defs/Waterfall"
      }
    },
    "subtotals": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Subtotal"
      },
      "description": "Per-period subtotals declared by the active pack, in dependency order. Omitted when the pack declares none."
    },
    "contracts": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Contract"
      }
    },
    "streams": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Stream"
      }
    },
    "stream_inputs": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/StreamInputs"
      },
      "description": "Per-stream record of what each pack rule consumed. Omitted when nothing was lowered from a pack."
    },
    "events": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Event"
      }
    },
    "options": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Option"
      }
    },
    "runs": {
      "type": "array",
      "minItems": 1,
      "items": {
        "$ref": "#/$defs/Run"
      }
    },
    "metrics": {
      "type": "array",
      "minItems": 0,
      "items": {
        "$ref": "#/$defs/Metric"
      },
      "description": "Reserved. Metrics are computed at run time by the engine and by the active pack, so a compile output does not carry them; no compiler emits this field."
    },
    "required_observables": {
      "type": "array",
      "description": "Ontology observable IDs referenced via obs('...')",
      "items": {
        "type": "string",
        "minLength": 1
      },
      "uniqueItems": true
    },
    "required_refs": {
      "type": "array",
      "description": "Ontology reference IDs referenced via ref('...')",
      "items": {
        "type": "string",
        "minLength": 1
      },
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
      "required": [
        "start",
        "end"
      ],
      "properties": {
        "start": {
          "$ref": "#/$defs/Date"
        },
        "end": {
          "$ref": "#/$defs/Date"
        }
      }
    },
    "Frequency": {
      "type": "string",
      "enum": [
        "daily",
        "weekly",
        "monthly",
        "quarterly",
        "annual"
      ]
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
    "Rate": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "value"
      ],
      "properties": {
        "value": {
          "$ref": "#/$defs/Decimal"
        },
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
      "required": [
        "lang",
        "src"
      ],
      "properties": {
        "lang": {
          "type": "string",
          "enum": [
            "cfdl"
          ]
        },
        "src": {
          "type": "string",
          "minLength": 1
        }
      }
    },
    "Phase": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "range",
        "name"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "range": {
          "$ref": "#/$defs/DateRange"
        },
        "name": {
          "$ref": "#/$defs/Id"
        }
      }
    },
    "EntityRef": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "symbol"
      ],
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
      "required": [
        "id",
        "symbol",
        "type",
        "fields"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "symbol": {
          "type": "string",
          "pattern": "^[A-Za-z_][A-Za-z0-9_]*\\.[A-Za-z_][A-Za-z0-9_]*$"
        },
        "type": {
          "$ref": "#/$defs/Qname"
        },
        "fields": {
          "type": "object",
          "description": "Field values declared in the entity's block, checked against the fields its ontology type declares. Literals here: a field stated with '=' is a fact about the thing. A field that moves carries an 'init'/'next' rule instead.",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "state": {
          "type": "object",
          "description": "Mutable runtime state fields (may be empty at compile time)",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "parent": {
          "type": "string",
          "description": "The entity this one is part of. ALWAYS OPTIONAL, and absent for most entities: hierarchy is available at every grain and required at none. A pool models collective behaviour with no loans under it; a building needs no units. The modeller chooses the grain, and the language does not prefer one."
        },
        "initial_state": {
          "type": "string",
          "description": "The lifecycle state this entity starts in, overriding its type's declared initial. Absent when the type declares no lifecycle. An entity WITH a lifecycle is always in exactly one of its states — there is no null state and no undeclared state, which is what makes a misspelled status a compile error rather than a wrong answer."
        },
        "rules": {
          "type": "object",
          "description": "Fields that MOVE, as recurrences owned by this entity. A field stated with '=' is a fact and lives in `fields`; a field with an 'init'/'next' rule lives here. A rule with no 'next' in source is written out as `next prev`, because a field with no rule holds.",
          "additionalProperties": {
            "type": "object",
            "required": [
              "init",
              "next"
            ],
            "additionalProperties": false,
            "properties": {
              "init": {
                "$ref": "#/$defs/Expr"
              },
              "next": {
                "$ref": "#/$defs/Expr"
              },
              "schedule": {
                "$ref": "#/$defs/Schedule"
              }
            },
            "description": "A field's recurrence. `schedule` is present only on a field a PACK emitted: it inherits the contract's payment rhythm, so a monthly-paying pool on a daily book compounds twelve times a year rather than 365. A field a modeller wrote has none and steps every period."
          }
        }
      }
    },
    "TypedValue": {
      "description": "Strongly-typed value union used for fields/terms/state. anyOf, not oneOf: the members overlap structurally — an Expr {lang, src} is also a valid Map of strings — so requiring exactly one match can never hold for an untagged union.",
      "anyOf": [
        {
          "type": "string"
        },
        {
          "type": "boolean"
        },
        {
          "type": "integer"
        },
        {
          "type": "number"
        },
        {
          "$ref": "#/$defs/Date"
        },
        {
          "$ref": "#/$defs/Money"
        },
        {
          "$ref": "#/$defs/Rate"
        },
        {
          "$ref": "#/$defs/Expr"
        },
        {
          "type": "array",
          "items": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        }
      ]
    },
    "Curve": {
      "type": "object",
      "required": [
        "name",
        "points"
      ],
      "additionalProperties": false,
      "properties": {
        "name": {
          "type": "string"
        },
        "interpolation": {
          "type": "string",
          "enum": [
            "step",
            "linear"
          ]
        },
        "points": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "object",
            "required": [
              "date",
              "value"
            ],
            "additionalProperties": false,
            "properties": {
              "date": {
                "type": "string"
              },
              "value": {
                "type": "number"
              }
            }
          }
        }
      }
    },
    "Assumptions": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "constants",
        "random"
      ],
      "properties": {
        "constants": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/AssumeConstant"
          }
        },
        "random": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/AssumeRandom"
          }
        }
      }
    },
    "AssumeConstant": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "expr",
        "type"
      ],
      "properties": {
        "name": {
          "$ref": "#/$defs/Id"
        },
        "expr": {
          "$ref": "#/$defs/Expr"
        },
        "type": {
          "$ref": "#/$defs/ValueTypeId"
        }
      }
    },
    "AssumeRandom": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "dist",
        "type"
      ],
      "properties": {
        "name": {
          "$ref": "#/$defs/Id"
        },
        "dist": {
          "$ref": "#/$defs/Distribution"
        },
        "type": {
          "$ref": "#/$defs/ValueTypeId"
        }
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
      "required": [
        "kind",
        "params"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "Normal",
            "LogNormal",
            "Uniform",
            "Triangular"
          ]
        },
        "params": {
          "type": "object",
          "additionalProperties": {
            "type": [
              "number",
              "string",
              "boolean"
            ]
          }
        },
        "clip": {
          "type": "array",
          "items": {
            "type": "number"
          },
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
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "type": {
          "$ref": "#/$defs/Qname"
        },
        "subject": {
          "$ref": "#/$defs/EntityRef"
        },
        "term": {
          "$ref": "#/$defs/DateRange"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        },
        "parties": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "tags": {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "terms": {
          "type": "object",
          "description": "Contract terms; pack may validate and type-check",
          "additionalProperties": {
            "$ref": "#/$defs/TypedValue"
          }
        },
        "effects": {
          "$ref": "#/$defs/Effects"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Effects": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "streams"
      ],
      "properties": {
        "streams": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Stream"
          }
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
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "owner": {
          "$ref": "#/$defs/EntityRef"
        },
        "direction": {
          "$ref": "#/$defs/Direction"
        },
        "currency": {
          "$ref": "#/$defs/Currency"
        },
        "category": {
          "type": "string",
          "description": "What this stream is economically (revenue, opex, debt_service, ...). Aggregation reads this rather than pattern-matching the name, so the meaning is declared once at the point of emission instead of being re-derived by every consumer. Must name a category the active pack declares (E5022). Absent when the stream is unclassified, which is legal and leaves it out of every category fold."
        },
        "schedule": {
          "$ref": "#/$defs/Schedule"
        },
        "amount": {
          "$ref": "#/$defs/Expr"
        },
        "active_when": {
          "description": "If omitted in source, compiler should emit true",
          "$ref": "#/$defs/Expr"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Direction": {
      "type": "string",
      "enum": [
        "inflow",
        "outflow"
      ]
    },
    "Schedule": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
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
        "on": {
          "$ref": "#/$defs/Date"
        },
        "every": {
          "$ref": "#/$defs/Frequency"
        },
        "from": {
          "$ref": "#/$defs/Date"
        },
        "to": {
          "$ref": "#/$defs/Date"
        },
        "on_rule": {
          "description": "Optional rule for day-of-month or weekday sets",
          "$ref": "#/$defs/OnRule"
        },
        "convention": {
          "type": "string",
          "enum": [
            "none",
            "following",
            "modified_following",
            "preceding",
            "modified_preceding"
          ]
        },
        "calendar": {
          "type": "string"
        },
        "phase": {
          "type": "string",
          "description": "Phase name for PhaseEnter/EveryPhase"
        },
        "except_dates": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "also_dates": {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "due": {
          "description": "Annuity due: payment falls at the start of each interval, as for rent. Omitted for an ordinary annuity, which pays at the interval's end and is the default. Determines how far a payment is discounted, not which period holds it — see 12_payment_timing.md.",
          "type": "boolean"
        },
        "at_period_end": {
          "description": "A one-shot (`on_date`) flow that settles at the END of the period its date falls in, rather than on the date itself. Set by a pack lowering rule for a DISPOSAL: a reversion is taken at the end of the holding period and so discounts the full n periods, where an acquisition settles on its date and does not. Omitted for the default. See 12_payment_timing.md.",
          "type": "boolean"
        },
        "mid": {
          "description": "Mid-period convention: the flow is discounted from halfway through the period that earned it, rather than from that period's end. A convention rather than a date, so it is half a period on every calendar — which is what separates it from a day rule. Standard in project finance and banker DCFs, on the reasoning that a period's cash arrives throughout it. Applies to a price struck at a point in time (a disposal, a terminal value) only if that price really does accrue, which it normally does not. Mutually exclusive with `due`, a day rule, and payment terms (E2109). Omitted for the default. See 12_payment_timing.md.",
          "type": "boolean"
        },
        "net_days": {
          "description": "Days between a flow being earned and its cash moving. Omitted when cash lands in the period that earned it.",
          "type": "integer",
          "minimum": 0
        },
        "net_months": {
          "description": "Months between a flow being earned and its cash moving, stepped by the calendar rather than as 30-day units.",
          "type": "integer",
          "minimum": 0
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "OnDate"
              }
            }
          },
          "then": {
            "required": [
              "on"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "Every"
              }
            }
          },
          "then": {
            "required": [
              "every",
              "from",
              "to"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "PhaseEnter"
              }
            }
          },
          "then": {
            "required": [
              "phase"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "EveryPhase"
              }
            }
          },
          "then": {
            "required": [
              "every",
              "phase"
            ]
          }
        }
      ]
    },
    "OnRule": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "DayOfMonth",
            "EndOfMonth"
          ]
        },
        "day": {
          "type": "integer",
          "minimum": 1,
          "maximum": 31
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "DayOfMonth"
              }
            }
          },
          "then": {
            "required": [
              "day"
            ]
          }
        }
      ]
    },
    "Weekday": {
      "type": "string",
      "enum": [
        "Mon",
        "Tue",
        "Wed",
        "Thu",
        "Fri",
        "Sat",
        "Sun"
      ]
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
    "Event": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "when",
        "actions",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "when": {
          "$ref": "#/$defs/Expr"
        },
        "actions": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Action"
          }
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Action": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
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
        "entity": {
          "$ref": "#/$defs/EntityRef"
        },
        "field": {
          "$ref": "#/$defs/Id"
        },
        "value": {
          "$ref": "#/$defs/TypedValue"
        },
        "stream": {
          "$ref": "#/$defs/Id"
        },
        "contract": {
          "$ref": "#/$defs/Id"
        },
        "option": {
          "$ref": "#/$defs/Id"
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "SetEntityField"
              }
            }
          },
          "then": {
            "required": [
              "entity",
              "field",
              "value"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "ActivateStream"
              }
            }
          },
          "then": {
            "required": [
              "stream"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "DeactivateStream"
              }
            }
          },
          "then": {
            "required": [
              "stream"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "ActivateContract"
              }
            }
          },
          "then": {
            "required": [
              "contract"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "DeactivateContract"
              }
            }
          },
          "then": {
            "required": [
              "contract"
            ]
          }
        },
        {
          "if": {
            "properties": {
              "kind": {
                "const": "ExerciseOption"
              }
            }
          },
          "then": {
            "required": [
              "option"
            ]
          }
        }
      ]
    },
    "Option": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "type",
        "exercise_when",
        "payoff",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "type": {
          "$ref": "#/$defs/Qname"
        },
        "exercisable_in_phase": {
          "$ref": "#/$defs/Id"
        },
        "exercise_when": {
          "$ref": "#/$defs/Expr"
        },
        "payoff": {
          "$ref": "#/$defs/Expr"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        },
        "owner": {
          "$ref": "#/$defs/EntityRef",
          "description": "The asset this option is written on. AN OPTION IS A CONTRACT WITH AN ELECTION, so it attaches to something the way every other contract does. Absent on an option written before options had owners; without one its payoff belongs to no entity and falls out of every per-entity total."
        },
        "parties": {
          "type": "array",
          "description": "Who the option is between, by role. The role is named by the contract TYPE rather than by the party, because the same party is lessor in one agreement and lender in another — the role belongs to the agreement.",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [
              "role",
              "entity"
            ],
            "properties": {
              "role": {
                "type": "string"
              },
              "entity": {
                "$ref": "#/$defs/EntityRef"
              }
            }
          }
        }
      }
    },
    "Run": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "kind"
      ],
      "properties": {
        "kind": {
          "type": "string",
          "enum": [
            "deterministic",
            "monte_carlo"
          ]
        },
        "trials": {
          "type": "integer",
          "minimum": 1
        },
        "seed": {
          "type": "integer",
          "minimum": 0
        }
      },
      "allOf": [
        {
          "if": {
            "properties": {
              "kind": {
                "const": "monte_carlo"
              }
            }
          },
          "then": {
            "required": [
              "trials",
              "seed"
            ]
          }
        }
      ]
    },
    "Metric": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "name",
        "expr",
        "provenance"
      ],
      "properties": {
        "id": {
          "$ref": "#/$defs/Id"
        },
        "name": {
          "$ref": "#/$defs/Id"
        },
        "expr": {
          "$ref": "#/$defs/Expr"
        },
        "provenance": {
          "$ref": "#/$defs/NodeProvenance"
        }
      }
    },
    "Provenance": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "sources",
        "compiler"
      ],
      "properties": {
        "sources": {
          "type": "array",
          "items": {
            "type": "string",
            "minLength": 1
          },
          "minItems": 1
        },
        "compiler": {
          "type": "object",
          "additionalProperties": false,
          "required": [
            "name",
            "version",
            "hash"
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
            "hash": {
              "type": "string",
              "minLength": 8
            },
            "notes": {
              "type": "array",
              "items": {
                "type": "string",
                "minLength": 1
              }
            }
          }
        }
      }
    },
    "NodeProvenance": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "source_file",
        "source_span"
      ],
      "properties": {
        "source_file": {
          "type": "string",
          "minLength": 1
        },
        "source_span": {
          "type": "object",
          "additionalProperties": false,
          "required": [
            "start_line",
            "start_col",
            "end_line",
            "end_col"
          ],
          "properties": {
            "start_line": {
              "type": "integer",
              "minimum": 1
            },
            "start_col": {
              "type": "integer",
              "minimum": 1
            },
            "end_line": {
              "type": "integer",
              "minimum": 1
            },
            "end_col": {
              "type": "integer",
              "minimum": 1
            }
          }
        },
        "notes": {
          "type": "string"
        },
        "generated_by": {
          "$ref": "#/$defs/GeneratedBy"
        }
      }
    },
    "GeneratedBy": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "pack",
        "rule_id"
      ],
      "properties": {
        "pack": {
          "$ref": "#/$defs/PackRef"
        },
        "rule_id": {
          "type": "string",
          "minLength": 1
        }
      }
    },
    "PackRef": {
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
        }
      }
    },
    "StreamInputs": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "stream",
        "contract",
        "terms"
      ],
      "description": "What a pack lowering rule CONSUMED to strike one stream: the placeholders its templates actually substituted, plus the rule defaults that filled a gap. Not the contract's whole term map — a contract lowers to several streams and each reads a different subset, so 'the contract's terms' is not an answer to 'what struck this line'. Pack, rule id and source span are already on the stream's own provenance and are not repeated here. Absent for hand-written streams, which no rule struck.",
      "properties": {
        "stream": {
          "$ref": "#/$defs/Id"
        },
        "contract": {
          "type": "string",
          "description": "The contract instance the rule matched, including any suffix."
        },
        "terms": {
          "type": "object",
          "additionalProperties": {
            "type": "string"
          },
          "description": "Resolved placeholder values, as the strings the templates substituted. Not coerced: a term's payload is text plus a span, which is the contract packs already work against."
        },
        "defaults_applied": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Keys the contract did not supply, filled from the rule's own defaults. Separated because 'the model said 0' and 'the pack assumed 0' are different facts, and a reader tracing a number needs to tell them apart."
        }
      }
    },
    "Subtotal": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "id",
        "kind",
        "op"
      ],
      "description": "A per-period subtotal: a named fold over the ledger, lowered from the active pack. Where a metric reduces to one lifetime scalar, this produces a value per period — the middle rows of a statement. Folds CATEGORIES by preference rather than stream names, so net operating income is everything under `operating.*` and nothing enumerates which streams those are. Array order is DEPENDENCY order: an entry may reference only ones before it, which makes a cycle unexpressible rather than merely rejected. A subtotal is a fold OF the cash and never counts as cash: it is excluded from model.total, model.npv, model.net_cash_flow and the per-stream annual rollup, by the same construction that keeps a field out of the cash.",
      "properties": {
        "id": {
          "type": "string",
          "description": "Output series key; must start with `domain.`."
        },
        "kind": {
          "enum": [
            "money",
            "number"
          ]
        },
        "op": {
          "enum": [
            "sum",
            "negated_sum",
            "cumulative",
            "negated_cumulative",
            "ratio"
          ],
          "description": "How the subtotal folds. `sum` and `negated_sum` total one period; `cumulative` and `negated_cumulative` carry a running total, which is how a stock is derived from a flow — principal paid to date, capital called to date. `ratio` divides two money subtotals."
        },
        "categories": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Category path selectors, e.g. `operating.revenue.*`."
        },
        "streams": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Stream-name selectors, for what a category cannot express."
        },
        "subtotals": {
          "type": "array",
          "items": {
            "type": "string"
          },
          "description": "Ids of subtotals declared earlier."
        },
        "numerator": {
          "type": "string"
        },
        "denominator": {
          "type": "string"
        },
        "formula": {
          "type": "string",
          "description": "Human-readable lineage, emitted verbatim."
        }
      }
    },
    "Waterfall": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "entity",
        "source",
        "steps"
      ],
      "properties": {
        "name": {
          "type": "string"
        },
        "entity": {
          "type": "string",
          "description": "The entity whose cash this allocates."
        },
        "schedule": {
          "$ref": "#/$defs/Schedule"
        },
        "source": {
          "$ref": "#/$defs/Expr"
        },
        "steps": {
          "type": "array",
          "minItems": 0,
          "items": {
            "$ref": "#/$defs/WaterfallStep"
          }
        }
      }
    },
    "WaterfallStep": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "name",
        "payee",
        "amount"
      ],
      "properties": {
        "name": {
          "type": "string"
        },
        "payee": {
          "type": "string",
          "description": "The entity this step pays."
        },
        "amount": {
          "$ref": "#/$defs/Expr",
          "description": "What the step is owed. `remaining`, `paid.<step>` and `owed.<step>` are bound on top of the ordinary expression environment."
        }
      }
    }
  }
}
```
