# CFDL v0.1 — Grammar (EBNF)

**File name:** `CFDL_v0_1_Grammar.ebnf`

> This grammar is intentionally pragmatic for v0.1: it captures the surface syntax needed by the Core spec. It is suitable as the basis for a hand-written parser or parser-generator input after minor adaptation (token rules, whitespace/comment handling).

---

## 1) Lexical tokens (informative)

Implementations MUST support:
- **Whitespace**: spaces, tabs, newlines (ignored except as separators)
- **Line comments**: `// ...` to end of line
- **Block comments**: `/* ... */`

Recommended tokens:
- `IDENT`      → `[A-Za-z_][A-Za-z0-9_]*`
- `QNAME`      → `IDENT ('.' IDENT)*`
- `STRING`     → `"..."` with escapes
- `INT`        → digits with optional `_` separators
- `DECIMAL`    → digits `.` digits with optional `_`
- `DATE`       → `YYYY-MM` or `YYYY-MM-DD`

---

## 2) Grammar

```ebnf
module          = { statement } ;

statement       = version_stmt
                | model_stmt
                | use_pack_stmt
                | import_stmt
                | time_stmt
                | phase_stmt
                | entity_stmt
                | assume_stmt
                | contract_stmt
                | stream_stmt
                | event_stmt
                | option_stmt
                | run_stmt
                | metric_stmt
                ;

// --- header / modules ---
version_stmt    = "version" number ;

model_stmt      = "model" string_lit model_attr* ;
model_attr      = "currency" IDENT ;

use_pack_stmt   = "use" "pack" string_lit "version" string_lit ;

import_stmt     = "import" string_lit [ "as" IDENT ] ;

// --- time ---
time_stmt       = "time" "calendar" frequency "from" date_lit "for" INT ;
frequency       = "daily" | "monthly" | "quarterly" | "annual" ;

phase_stmt      = "phase" IDENT "from" date_lit "to" date_lit ;

// --- entities ---
entity_stmt     = "entity" IDENT IDENT ":" qname entity_block ;
entity_block    = "{" { kv_stmt } "}" ;
kv_stmt         = IDENT literal_or_expr ;

// --- assumptions ---
assume_stmt     = "assume" IDENT ( "=" expr | "~" dist_expr ) ;

dist_expr       = dist_name "(" [ dist_arg { "," dist_arg } ] ")" ;
dist_name       = "Normal" | "LogNormal" | "Uniform" | "Triangular" ;
dist_arg        = IDENT "=" literal
                | "clip" "=" list_number
                ;

// --- contracts ---
contract_stmt   = "contract" qname IDENT
                  "on" "entity" entity_ref
                  "term" date_lit ".." date_lit
                  contract_block ;

contract_block  = "{" { contract_item } "}" ;

contract_item   = currency_stmt
                | terms_block
                | effects_block
                | parties_block
                | tags_block
                ;

currency_stmt   = "currency" IDENT ;

terms_block     = "terms" map_block ;
parties_block   = "parties" map_block ;
tags_block      = "tags" map_block ;

map_block       = "{" { map_entry } "}" ;
map_entry       = IDENT literal_or_expr ;

// --- effects (streams in v0.1 core) ---
effects_block   = "effects" "{" { effect_stmt } "}" ;
effect_stmt     = stream_effect_stmt ;

stream_effect_stmt = "stream" IDENT
                     "owner" "entity" entity_ref
                     "direction" direction
                     "currency" IDENT
                     stream_block ;

direction       = "inflow" | "outflow" ;

// --- standalone streams ---
stream_stmt     = "stream" IDENT
                  "on" "entity" entity_ref
                  direction
                  "currency" IDENT
                  stream_block ;

stream_block    = "{" { stream_item } "}" ;
stream_item     = schedule_stmt
                | amount_stmt
                | active_stmt
                ;

active_stmt     = "active" "when" expr ;
amount_stmt     = "amount" expr ;

// --- schedules ---
schedule_stmt   = "schedule" schedule_expr ;

schedule_expr   = schedule_on
                | schedule_every
                | schedule_phase_enter
                | schedule_every_phase
                ;

schedule_on     = "on" date_lit ;

schedule_phase_enter = "on" "phase_enter" "(" string_lit ")" ;

schedule_every_phase = "every" frequency
                       "from" "phase_start" "(" string_lit ")"
                       "to" "phase_end" "(" string_lit ")"
                       [ schedule_opts ] ;

schedule_every  = "every" frequency
                  [ schedule_on_day ]
                  "from" date_lit "to" date_lit
                  [ schedule_opts ] ;

schedule_on_day = "on" ( "day" INT | "eom" | weekday_list ) ;
weekday_list    = weekday { "," weekday } ;
weekday         = "Mon" | "Tue" | "Wed" | "Thu" | "Fri" | "Sat" | "Sun" ;

schedule_opts   = { schedule_opt } ;

schedule_opt    = "convention" convention
                | "calendar" string_lit
                | "stub" stub_policy
                | "except" list_date
                | "also" list_date
                ;

convention      = "none" | "following" | "modified_following" | "preceding" | "modified_preceding" ;
stub_policy     = "none" | "short_front" | "short_back" | "long_front" | "long_back" ;

list_date       = "[" date_lit { "," date_lit } "]" ;

// --- events ---
event_stmt      = "event" IDENT "when" expr event_block ;
event_block     = "{" { action_stmt } "}" ;

action_stmt     = set_entity_stmt
                | activate_stream_stmt
                | deactivate_stream_stmt
                | activate_contract_stmt
                | deactivate_contract_stmt
                | exercise_option_stmt
                ;

set_entity_stmt = "set" "entity" entity_ref "." IDENT "=" literal_or_expr ;

activate_stream_stmt   = "activate" "stream" IDENT ;
deactivate_stream_stmt = "deactivate" "stream" IDENT ;

activate_contract_stmt   = "activate" "contract" IDENT ;
deactivate_contract_stmt = "deactivate" "contract" IDENT ;

exercise_option_stmt   = "exercise" "option" IDENT ;

// --- options ---
option_stmt     = "option" IDENT
                  "type" qname
                  [ "exercisable" "in" IDENT ]
                  option_block ;

option_block    = "{" option_item* "}" ;
option_item     = "exercise" "when" expr
                | "payoff" expr
                ;

// --- runs & metrics ---
run_stmt        = "run" ( "deterministic" | mc_run ) ;
mc_run          = "monte_carlo" "trials" INT "seed" INT ;

metric_stmt     = "metric" IDENT "=" expr ;

// --- expressions & literals ---
expr            = "cel" string_lit ;

literal_or_expr = literal | expr ;

literal         = string_lit | number | bool_lit | date_lit | money_lit | list | map_inline ;

money_lit       = number IDENT ;  // e.g., 42000 USD

list            = "[" [ literal { "," literal } ] "]" ;

map_inline      = "{" [ IDENT literal_or_expr { IDENT literal_or_expr } ] "}" ;

list_number     = "[" number { "," number } "]" ;

entity_ref      = IDENT "." IDENT ;
qname           = IDENT { "." IDENT } ;

string_lit      = STRING ;
number          = DECIMAL | INT ;
bool_lit        = "true" | "false" ;
date_lit        = DATE ;

```

---

## 3) Notes & implementation guidance (informative)

1. **Line breaks** are not significant; blocks and keywords provide structure.
2. `money_lit` is syntactic sugar; implementations should normalize to `Money` with `currency`.
3. `DATE` accepts `YYYY-MM` and `YYYY-MM-DD`; normalize `YYYY-MM` to `YYYY-MM-01` during parsing.
4. `schedule_opts` are order-independent; parsers should accept any order.
5. `activate contract` / `deactivate contract` are included in grammar, but may be a no-op in early engines.

