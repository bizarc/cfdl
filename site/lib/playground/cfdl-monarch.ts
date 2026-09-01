import type { languages } from "monaco-editor";

export const CFDL_LANGUAGE_ID = "cfdl";

/**
 * Monaco tokenizer for CFDL.
 *
 * Monaco cannot consume TextMate grammars without a heavy runtime, so the
 * editor needs its own tokenizer. The vocabulary below is transcribed from
 * `editors/vscode/syntaxes/cfdl.tmLanguage.json` — the grammar that the VS
 * Code extension and the docs site (via Shiki) both use — so all three agree
 * on what a keyword is. When the grammar's keyword list changes, update this
 * list to match.
 */
const KEYWORDS = [
  "version", "model", "use", "pack", "import", "as", "time", "calendar",
  "from", "for", "to", "phase", "entity", "assume", "contract", "on", "term",
  "terms", "effects", "parties", "tags", "stream", "owner", "direction",
  "inflow", "outflow", "schedule", "every", "phase_enter", "phase_start",
  "phase_end", "day", "eom", "convention", "stub", "except", "also", "event",
  "when", "set", "activate", "deactivate", "exercise", "option", "type",
  "exercisable", "in", "payoff", "run", "metric", "active", "and", "or",
  "not", "currency", "curve", "slice", "statement", "waterfall", "account",
  "quantile", "state",
];

const FREQUENCIES = ["daily", "monthly", "quarterly", "annual"];
const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const CONVENTIONS = [
  "none", "following", "modified_following", "preceding",
  "modified_preceding", "short_front", "short_back", "long_front", "long_back",
];
const RUN_KEYWORDS = ["deterministic", "monte_carlo", "trials", "seed"];
const DISTRIBUTIONS = ["Normal", "LogNormal", "Uniform", "Triangular", "clip"];

export const CFDL_MONARCH: languages.IMonarchLanguage = {
  defaultToken: "",
  keywords: KEYWORDS,
  frequencies: FREQUENCIES,
  weekdays: WEEKDAYS,
  conventions: CONVENTIONS,
  runKeywords: RUN_KEYWORDS,
  distributions: DISTRIBUTIONS,

  tokenizer: {
    root: [
      [/\/\/.*$/, "comment"],
      [/\/\*/, "comment", "@comment"],

      // Dates before numbers, so 2026-01 isn't lexed as arithmetic.
      [/\b\d{4}-\d{2}(-\d{2})?\b/, "number.hex"],
      [/\b\d[\d_]*(\.[\d_]+)?\b/, "number"],

      [/"/, "string", "@string"],

      [
        /[A-Za-z_][\w.]*/,
        {
          cases: {
            "@keywords": "keyword",
            "@frequencies": "keyword",
            "@weekdays": "keyword",
            "@conventions": "keyword",
            "@runKeywords": "keyword",
            "@distributions": "type",
            "true|false": "constant",
            "@default": "identifier",
          },
        },
      ],

      [/[{}()[\]]/, "@brackets"],
      [/[=<>!~?:&|+\-*/^%]+/, "operator"],
    ],

    comment: [
      [/[^/*]+/, "comment"],
      [/\*\//, "comment", "@pop"],
      [/[/*]/, "comment"],
    ],

    string: [
      [/[^\\"]+/, "string"],
      [/\\./, "string.escape"],
      [/"/, "string", "@pop"],
    ],
  },
};
