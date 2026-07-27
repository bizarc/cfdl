import { compressToEncodedURIComponent, decompressFromEncodedURIComponent } from "lz-string";
import type { RunConfig } from "./protocol";

export interface SharedState {
  v: 1;
  files: Record<string, string>;
  root: string;
  config?: RunConfig;
  pack?: string;
}

const HASH_KEY = "code";
const DRAFT_KEY = "cfdl.playground.draft.v1";

/** Compressed into the URL fragment, so a shared model never touches a server. */
export function encodeShare(state: Omit<SharedState, "v">): string {
  return compressToEncodedURIComponent(JSON.stringify({ v: 1, ...state }));
}

export function decodeShare(encoded: string): SharedState | null {
  try {
    const json = decompressFromEncodedURIComponent(encoded);
    if (!json) return null;
    const parsed = JSON.parse(json) as SharedState;
    if (parsed.v !== 1 || typeof parsed.files !== "object" || !parsed.root) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function shareUrl(state: Omit<SharedState, "v">, origin = ""): string {
  const base = origin || (typeof window !== "undefined" ? window.location.origin : "");
  return `${base}/playground#${HASH_KEY}=${encodeShare(state)}`;
}

export function readShareFromHash(): SharedState | null {
  if (typeof window === "undefined") return null;
  const hash = window.location.hash.replace(/^#/, "");
  if (!hash.startsWith(`${HASH_KEY}=`)) return null;
  return decodeShare(hash.slice(HASH_KEY.length + 1));
}

export function saveDraft(state: Omit<SharedState, "v">) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(DRAFT_KEY, JSON.stringify({ v: 1, ...state }));
  } catch {
    // Private browsing or a full quota — drafts are a convenience, not a
    // guarantee, so losing them must never break the editor.
  }
}

export function readDraft(): SharedState | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.localStorage.getItem(DRAFT_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as SharedState;
    return parsed.v === 1 && parsed.files ? parsed : null;
  } catch {
    return null;
  }
}

export function clearDraft() {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.removeItem(DRAFT_KEY);
  } catch {
    /* ignore */
  }
}
