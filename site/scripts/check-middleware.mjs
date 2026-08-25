#!/usr/bin/env node
/**
 * Gate for the private-page credential check.
 *
 * `middleware.ts` decides whether an unauthenticated request reaches a private
 * page. Next fails no build over a wrong answer here, and the failure mode is
 * silent: a page that should challenge simply serves. So the decision is
 * asserted directly, the way check-links stands in for a broken-link check.
 *
 * Bare Node, no test runner — the logic lives in lib/basic-auth.mjs precisely
 * so this file can import it.
 */
import { isAuthorized, safeEqual } from "../lib/basic-auth.mjs";

const USER = "team";
const PASS = "correct horse battery staple";

/** @param {string} u @param {string} p */
const header = (u, p) => `Basic ${btoa(`${u}:${p}`)}`;

let failures = 0;

/** @param {string} name @param {boolean} actual @param {boolean} expected */
function check(name, actual, expected) {
  if (actual === expected) return;
  failures += 1;
  console.error(`  FAIL  ${name}\n        expected ${expected}, got ${actual}`);
}

// --- the one case that must pass ------------------------------------------
check("correct credentials", isAuthorized(header(USER, PASS), USER, PASS), true);
check(
  "scheme is case-insensitive (RFC 7235)",
  isAuthorized(header(USER, PASS).replace("Basic", "basic"), USER, PASS),
  true,
);
check(
  "a password containing a colon survives",
  isAuthorized(header(USER, "a:b:c"), USER, "a:b:c"),
  true,
);

// --- everything else must be refused --------------------------------------
check("wrong password", isAuthorized(header(USER, "wrong"), USER, PASS), false);
check("wrong user", isAuthorized(header("nobody", PASS), USER, PASS), false);
check("empty password", isAuthorized(header(USER, ""), USER, PASS), false);
check("no header", isAuthorized(null, USER, PASS), false);
check("empty header", isAuthorized("", USER, PASS), false);
check("no scheme", isAuthorized(btoa(`${USER}:${PASS}`), USER, PASS), false);
check("bearer token", isAuthorized(`Bearer ${btoa(`${USER}:${PASS}`)}`, USER, PASS), false);
check("undecodable base64", isAuthorized("Basic !!!not-base64!!!", USER, PASS), false);
check("no colon in payload", isAuthorized(`Basic ${btoa("nocolon")}`, USER, PASS), false);
check("empty credential payload", isAuthorized("Basic ", USER, PASS), false);

// A prefix of the real password must not pass. This is the bug a `startsWith`
// or a truncating comparison would introduce, and it is invisible by
// inspection once the happy path works.
check(
  "password prefix is refused",
  isAuthorized(header(USER, PASS.slice(0, -1)), USER, PASS),
  false,
);
check(
  "password with trailing byte is refused",
  isAuthorized(header(USER, `${PASS}x`), USER, PASS),
  false,
);

// --- unconfigured deployment authorizes nobody -----------------------------
// The middleware turns this into a 503; what matters here is that a missing
// secret can never be the thing that lets a request through.
check("no user configured", isAuthorized(header(USER, PASS), undefined, PASS), false);
check("no password configured", isAuthorized(header(USER, PASS), USER, undefined), false);
check("neither configured", isAuthorized(header(USER, PASS), undefined, undefined), false);
check("empty configured password", isAuthorized(header(USER, ""), USER, ""), false);

// --- the comparison itself -------------------------------------------------
check("safeEqual: equal", safeEqual("abc", "abc"), true);
check("safeEqual: differing length", safeEqual("abc", "abcd"), false);
check("safeEqual: same length, differing byte", safeEqual("abc", "abd"), false);
check("safeEqual: empty strings", safeEqual("", ""), true);
check("safeEqual: non-string", safeEqual("abc", undefined), false);

if (failures > 0) {
  console.error(`\ncheck-middleware: ${failures} check(s) failed`);
  process.exit(1);
}

console.log("check-middleware: OK (private pages challenge every unauthenticated request)");
