/**
 * Message contract between the page and the engine worker.
 *
 * The wasm engine is synchronous, so every call would block whatever thread
 * it runs on. Keeping it in a worker means a 10,000-trial Monte Carlo can't
 * freeze typing or scrolling, and "stop" is implementable (by terminating
 * and respawning the worker).
 */

export interface Diagnostic {
  code: string;
  severity: "error" | "warning" | "info";
  message: string;
  file?: string;
  span?: {
    start_line: number;
    start_col: number;
    end_line: number;
    end_col: number;
  };
  hint?: string;
  notes?: string[];
}

/** Run-config JSON, matching cfdl-engine's RunConfigFile. */
export interface RunConfig {
  deterministic?: {
    annual_discount_rate?: number;
    as_of?: string;
    parameters?: Record<string, number>;
  };
  scenarios?: Record<
    string,
    {
      annual_discount_rate?: number;
      as_of?: string;
      parameters?: Record<string, number>;
    }
  >;
  monte_carlo?: {
    trial_count: number;
    seed: number;
    distributions?: Record<string, Record<string, unknown>>;
  };
}

export type WorkerRequest =
  | { id: number; type: "init" }
  | {
      id: number;
      type: "run";
      files: Record<string, string>;
      root: string;
      config?: RunConfig;
      pack?: string;
    }
  | {
      id: number;
      type: "compile";
      files: Record<string, string>;
      root: string;
    };

export type WorkerResponse =
  | { id: number; type: "ready" }
  | { id: number; type: "ok"; results: unknown }
  | { id: number; type: "compiled"; ir: unknown }
  | { id: number; type: "diagnostics"; diagnostics: Diagnostic[] }
  | { id: number; type: "error"; message: string };

/** What the engine returns from compile/run, before we narrow it. */
export type EngineEnvelope =
  | { ok: true; results?: unknown; ir?: unknown }
  | { ok: false; diagnostics?: Diagnostic[]; error?: string };
