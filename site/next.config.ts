import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { NextConfig } from "next";

/**
 * Cache key for the wasm engine.
 *
 * The playground loads /wasm/cfdl_wasm.js and its .wasm by fixed URL, so a
 * returning visitor can keep a stale engine in HTTP cache long after a fresh
 * one deploys — the same class of failure as a stale committed bundle, just
 * one layer out. The build stamp is a hash of the engine and pack sources, so
 * it changes exactly when the bundle does and never otherwise.
 */
function wasmBuildId(): string {
  try {
    return readFileSync(join(process.cwd(), "public", "wasm", ".build-stamp"), "utf8")
      .trim()
      .slice(0, 12);
  } catch {
    // A missing stamp is caught by `npm run check:wasm`; don't fail the build
    // here, just fall back to a value that disables caching benefits.
    return "dev";
  }
}

const nextConfig: NextConfig = {
  env: {
    NEXT_PUBLIC_WASM_BUILD: wasmBuildId(),
  },
};

export default nextConfig;
