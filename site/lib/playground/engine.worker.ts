/// <reference lib="webworker" />
import type {
  EngineEnvelope,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";

type WasmModule = {
  default: (input?: unknown) => Promise<unknown>;
  compile: (filesJson: string, rootFile: string) => string;
  run: (irJson: string, configJson?: string, pack?: string) => string;
  compile_and_run: (
    filesJson: string,
    rootFile: string,
    configJson?: string,
    pack?: string,
  ) => string;
};

let wasm: WasmModule | null = null;
let loading: Promise<WasmModule> | null = null;

/**
 * The glue is emitted by wasm-bindgen into public/wasm and served as a static
 * asset, not bundled. Building the specifier at runtime keeps the bundler from
 * trying to resolve and inline it.
 */
async function loadWasm(): Promise<WasmModule> {
  if (wasm) return wasm;
  if (!loading) {
    loading = (async () => {
      const base = self.location.origin;
      // Both URLs are fixed, so a returning visitor would otherwise keep a
      // cached engine after a new one deploys. The build id is a hash of the
      // engine and pack sources: it changes exactly when the bundle does.
      const build = process.env.NEXT_PUBLIC_WASM_BUILD ?? "dev";
      const glueUrl = `${base}/wasm/cfdl_wasm.js?v=${build}`;
      const binaryUrl = `${base}/wasm/cfdl_wasm_bg.wasm?v=${build}`;
      const mod = (await import(/* webpackIgnore: true */ glueUrl)) as WasmModule;
      await mod.default(binaryUrl);
      wasm = mod;
      return mod;
    })();
  }
  return loading;
}

function post(message: WorkerResponse) {
  (self as unknown as Worker).postMessage(message);
}

function respondToEnvelope(id: number, raw: string, kind: "run" | "compile") {
  const envelope = JSON.parse(raw) as EngineEnvelope;

  if (envelope.ok) {
    if (kind === "run") post({ id, type: "ok", results: envelope.results });
    else post({ id, type: "compiled", ir: envelope.ir });
    return;
  }

  if (envelope.diagnostics?.length) {
    post({ id, type: "diagnostics", diagnostics: envelope.diagnostics });
  } else {
    post({ id, type: "error", message: envelope.error ?? "Unknown engine error" });
  }
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const request = event.data;

  try {
    const engine = await loadWasm();

    switch (request.type) {
      case "init":
        post({ id: request.id, type: "ready" });
        break;

      case "run":
        respondToEnvelope(
          request.id,
          engine.compile_and_run(
            JSON.stringify(request.files),
            request.root,
            request.config ? JSON.stringify(request.config) : undefined,
            request.pack,
          ),
          "run",
        );
        break;

      case "compile":
        respondToEnvelope(
          request.id,
          engine.compile(JSON.stringify(request.files), request.root),
          "compile",
        );
        break;
    }
  } catch (error) {
    post({
      id: request.id,
      type: "error",
      message: error instanceof Error ? error.message : String(error),
    });
  }
};
