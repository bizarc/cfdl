import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { NextConfig } from "next";

/**
 * Same cache-busting stamp the site uses: the engine worker fetches
 * /wasm/cfdl_wasm.js by fixed URL, so the build id — a hash of the engine
 * sources — is what keeps a returning visitor off a stale engine.
 */
function wasmBuildId(): string {
  try {
    return readFileSync(join(process.cwd(), "public", "wasm", ".build-stamp"), "utf8")
      .trim()
      .slice(0, 12);
  } catch {
    return "dev";
  }
}

const nextConfig: NextConfig = {
  env: {
    // "Open in playground" on a code block deep-links into the site's
    // playground; the model travels in the URL fragment.
    NEXT_PUBLIC_PLAYGROUND_ORIGIN: "https://cfdl.dev",
    NEXT_PUBLIC_WASM_BUILD: wasmBuildId(),
  },
};

export default nextConfig;
