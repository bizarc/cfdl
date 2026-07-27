import type {
  Diagnostic,
  RunConfig,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";

export type RunOutcome =
  | { status: "ok"; results: unknown }
  | { status: "diagnostics"; diagnostics: Diagnostic[] }
  | { status: "error"; message: string };

type Pending = {
  resolve: (outcome: RunOutcome) => void;
};

/**
 * Owns the engine worker and turns its message stream into promises.
 *
 * Cancellation is a terminate-and-respawn: the wasm engine is synchronous, so
 * a long Monte Carlo cannot be interrupted cooperatively. Dropping the worker
 * is the only way to stop it, and the next call transparently starts a new one.
 */
export class EngineClient {
  private worker: Worker | null = null;
  private pending = new Map<number, Pending>();
  private nextId = 1;
  private readyPromise: Promise<void> | null = null;

  private spawn(): Worker {
    const worker = new Worker(new URL("./engine.worker.ts", import.meta.url), {
      type: "module",
    });

    worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const message = event.data;
      const entry = this.pending.get(message.id);
      if (!entry) return;
      this.pending.delete(message.id);

      switch (message.type) {
        case "ready":
          entry.resolve({ status: "ok", results: null });
          break;
        case "ok":
          entry.resolve({ status: "ok", results: message.results });
          break;
        case "compiled":
          entry.resolve({ status: "ok", results: message.ir });
          break;
        case "diagnostics":
          entry.resolve({ status: "diagnostics", diagnostics: message.diagnostics });
          break;
        case "error":
          entry.resolve({ status: "error", message: message.message });
          break;
      }
    };

    // A wasm trap kills the worker outright; fail every in-flight call rather
    // than leaving callers hanging, and let the next call respawn.
    worker.onerror = () => {
      for (const [, entry] of this.pending) {
        entry.resolve({
          status: "error",
          message: "The engine stopped unexpectedly. Try running again.",
        });
      }
      this.pending.clear();
      this.worker = null;
      this.readyPromise = null;
    };

    return worker;
  }

  private ensureWorker(): Worker {
    if (!this.worker) this.worker = this.spawn();
    return this.worker;
  }

  private send(request: Omit<WorkerRequest, "id">): Promise<RunOutcome> {
    const worker = this.ensureWorker();
    const id = this.nextId++;
    return new Promise<RunOutcome>((resolve) => {
      this.pending.set(id, { resolve });
      worker.postMessage({ ...request, id } as WorkerRequest);
    });
  }

  /**
   * Starts the worker and compiles the wasm module. Called on mount so the
   * engine is warm before the reader asks for anything.
   */
  ready(): Promise<void> {
    if (!this.readyPromise) {
      this.readyPromise = this.send({ type: "init" }).then(() => undefined);
    }
    return this.readyPromise;
  }

  run(args: {
    files: Record<string, string>;
    root: string;
    config?: RunConfig;
    pack?: string;
  }): Promise<RunOutcome> {
    return this.send({ type: "run", ...args });
  }

  compile(args: { files: Record<string, string>; root: string }): Promise<RunOutcome> {
    return this.send({ type: "compile", ...args });
  }

  /** Stops an in-flight run. The next call spawns a fresh worker. */
  cancel() {
    if (!this.worker) return;
    this.worker.terminate();
    for (const [, entry] of this.pending) {
      entry.resolve({ status: "error", message: "Run cancelled." });
    }
    this.pending.clear();
    this.worker = null;
    this.readyPromise = null;
  }

  dispose() {
    this.cancel();
  }
}

/** One engine per tab; docs cells and the playground share it. */
let shared: EngineClient | null = null;

export function getEngineClient(): EngineClient {
  if (!shared) shared = new EngineClient();
  return shared;
}
