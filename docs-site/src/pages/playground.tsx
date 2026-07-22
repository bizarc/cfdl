import React, {useCallback, useEffect, useRef, useState} from 'react';
import Layout from '@theme/Layout';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useBaseUrl from '@docusaurus/useBaseUrl';

// Minimal Monarch tokenizer ported from the VS Code TextMate grammar. Kept
// deliberately small; the authoritative grammar lives in editors/vscode.
const CFDL_MONARCH = {
  keywords: [
    'version', 'model', 'use', 'pack', 'import', 'as', 'time', 'calendar',
    'from', 'for', 'project', 'monthly', 'quarterly', 'annual', 'daily',
    'entity', 'assume', 'curve', 'contract', 'on', 'term', 'terms', 'stream',
    'schedule', 'every', 'amount', 'active', 'when', 'event', 'option', 'run',
    'deterministic', 'monte_carlo', 'trials', 'seed', 'inflow', 'outflow',
    'currency', 'phase', 'step', 'linear',
  ],
  tokenizer: {
    root: [
      [/\/\/.*$/, 'comment'],
      [/"[^"]*"/, 'string'],
      [/\b\d{4}-\d{2}(-\d{2})?\b/, 'number'],
      [/\b\d+(\.\d+)?\b/, 'number'],
      [/[a-zA-Z_][\w.]*/, {
        cases: {'@keywords': 'keyword', '@default': 'identifier'},
      }],
    ],
  },
};

const STARTER = `version 0.1
model "playground"
use pack "credit" version "0.1.0"
time calendar monthly from 2026-01 for 12

entity fund buyer

contract credit.pool_level_pay.a on entity fund.buyer {
  term 2026-01..2026-12
  terms {
    balance = 1000000
    rate = 0.07
    term_months = 12
    cpr = 0.08
    cdr = 0.02
    severity = 0.35
  }
}

contract credit.purchase.a on entity fund.buyer {
  term 2026-01..2026-01
  terms { price = 1000000 }
}
`;

type WasmModule = {
  default: (path: string) => Promise<unknown>;
  compile_and_run: (
    filesJson: string,
    rootFile: string,
    configJson?: string,
    pack?: string,
  ) => string;
};

function PlaygroundInner() {
  const wasmJsUrl = useBaseUrl('/wasm/cfdl_wasm.js');
  const wasmBgUrl = useBaseUrl('/wasm/cfdl_wasm_bg.wasm');
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [source, setSource] = useState(STARTER);
  const [pack, setPack] = useState('credit');
  const [output, setOutput] = useState('Loading engine…');
  const editorRef = useRef<any>(null);
  const monacoRef = useRef<any>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const mod = (await import(/* webpackIgnore: true */ wasmJsUrl)) as WasmModule;
        await mod.default(wasmBgUrl);
        if (!cancelled) {
          setWasm(mod);
          setOutput('Engine ready. Press Run.');
        }
      } catch (err: any) {
        if (!cancelled) setLoadError(String(err?.message ?? err));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [wasmJsUrl, wasmBgUrl]);

  const setMarkers = useCallback((diagnostics: any[]) => {
    const editor = editorRef.current;
    const monaco = monacoRef.current;
    if (!editor || !monaco) return;
    const markers = (diagnostics ?? []).map((d) => {
      const span = d.span ?? {};
      return {
        severity: monaco.MarkerSeverity.Error,
        message: `${d.code}: ${d.message}`,
        startLineNumber: span.start_line ?? 1,
        startColumn: span.start_col ?? 1,
        endLineNumber: span.end_line ?? span.start_line ?? 1,
        endColumn: (span.end_col ?? span.start_col ?? 1) + 1,
      };
    });
    monaco.editor.setModelMarkers(editor.getModel(), 'cfdl', markers);
  }, []);

  const runModel = useCallback(() => {
    if (!wasm) return;
    setMarkers([]);
    let result: any;
    try {
      const files = JSON.stringify({'model.cfdl': source});
      const raw = wasm.compile_and_run(files, 'model.cfdl', undefined, pack || undefined);
      result = JSON.parse(raw);
    } catch (err: any) {
      setOutput(`Runtime error: ${err?.message ?? err}`);
      return;
    }
    if (result.ok === false) {
      if (result.diagnostics) {
        setMarkers(result.diagnostics);
        setOutput(
          'Diagnostics:\n' +
            result.diagnostics
              .map((d: any) => `  ${d.code}: ${d.message}`)
              .join('\n'),
        );
      } else {
        setOutput(`Error: ${result.error}`);
      }
      return;
    }
    const r = result.results;
    const metrics = r.deterministic?.metrics ?? {};
    const domain = r.domain_metrics?.metrics ?? {};
    const fmt = (v: any) => (v && typeof v === 'object' && 'amount' in v ? v.amount : v);
    const lines = ['Metrics:'];
    for (const [k, v] of Object.entries(metrics)) lines.push(`  ${k} = ${fmt(v)}`);
    if (Object.keys(domain).length) {
      lines.push('', 'Domain metrics:');
      for (const [k, v] of Object.entries(domain)) lines.push(`  ${k} = ${fmt(v)}`);
    }
    setOutput(lines.join('\n'));
  }, [wasm, source, pack, setMarkers]);

  return (
    <div style={{display: 'flex', flexDirection: 'column', gap: 12, padding: 16}}>
      <div style={{display: 'flex', gap: 12, alignItems: 'center'}}>
        <button
          className="button button--primary"
          onClick={runModel}
          disabled={!wasm}>
          Run
        </button>
        <label>
          pack:{' '}
          <input
            value={pack}
            onChange={(e) => setPack(e.target.value)}
            placeholder="(none)"
            style={{width: 100}}
          />
        </label>
        {loadError && (
          <span style={{color: 'var(--ifm-color-danger)'}}>
            Engine failed to load: {loadError}
          </span>
        )}
      </div>
      <div style={{display: 'flex', gap: 12, minHeight: 480}}>
        <div style={{flex: 1, border: '1px solid var(--ifm-color-emphasis-300)'}}>
          <MonacoLoader
            source={source}
            onChange={setSource}
            onMount={(editor: any, monaco: any) => {
              editorRef.current = editor;
              monacoRef.current = monaco;
              monaco.languages.register({id: 'cfdl'});
              monaco.languages.setMonarchTokensProvider('cfdl', CFDL_MONARCH as any);
              monaco.editor.setModelLanguage(editor.getModel(), 'cfdl');
            }}
          />
        </div>
        <pre
          style={{
            flex: 1,
            margin: 0,
            padding: 12,
            overflow: 'auto',
            background: 'var(--ifm-code-background)',
            border: '1px solid var(--ifm-color-emphasis-300)',
          }}>
          {output}
        </pre>
      </div>
    </div>
  );
}

function MonacoLoader({source, onChange, onMount}: any) {
  const [Editor, setEditor] = useState<any>(null);
  useEffect(() => {
    import('@monaco-editor/react').then((m) => setEditor(() => m.default));
  }, []);
  if (!Editor) return <div style={{padding: 12}}>Loading editor…</div>;
  return (
    <Editor
      height="480px"
      defaultLanguage="cfdl"
      value={source}
      onChange={(v: string | undefined) => onChange(v ?? '')}
      onMount={onMount}
      options={{minimap: {enabled: false}, fontSize: 13}}
    />
  );
}

export default function Playground() {
  return (
    <Layout title="Playground" description="Compile and run CFDL in your browser">
      <BrowserOnly fallback={<div style={{padding: 16}}>Loading playground…</div>}>
        {() => <PlaygroundInner />}
      </BrowserOnly>
    </Layout>
  );
}
