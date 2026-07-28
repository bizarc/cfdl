import fs from "node:fs";
import path from "node:path";
import {
  createHighlighter,
  type Highlighter,
  type LanguageRegistration,
} from "shiki";

/**
 * The site highlights CFDL with the *same* TextMate grammar the VS Code
 * extension ships, so code on cfdl.dev is tokenized identically to code in
 * the editor. There is no second grammar to keep in sync.
 */
const GRAMMAR_PATH = path.join(
  process.cwd(),
  "..",
  "editors",
  "vscode",
  "syntaxes",
  "cfdl.tmLanguage.json",
);

function loadCfdlGrammar(): LanguageRegistration {
  const raw = JSON.parse(fs.readFileSync(GRAMMAR_PATH, "utf8"));
  // `name` becomes the language id; an alias equal to it would self-reference.
  return { ...raw, name: "cfdl" } as LanguageRegistration;
}

let highlighterPromise: Promise<Highlighter> | null = null;

export function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: ["github-light", "github-dark-default"],
      langs: [
        loadCfdlGrammar(),
        "bash",
        "json",
        "toml",
        "python",
        "rust",
        "typescript",
      ],
    });
  }
  return highlighterPromise;
}

/**
 * Renders to dual-theme HTML: both palettes ride along as CSS variables and
 * `globals.css` picks one from `[data-theme]` — no re-render on theme flip,
 * no flash.
 */
export async function highlight(code: string, lang = "cfdl"): Promise<string> {
  const highlighter = await getHighlighter();
  const known = highlighter.getLoadedLanguages();
  const language = known.includes(lang) ? lang : "text";

  return highlighter.codeToHtml(code.trimEnd(), {
    lang: language,
    themes: { light: "github-light", dark: "github-dark-default" },
    defaultColor: false,
    cssVariablePrefix: "--shiki-",
  });
}
