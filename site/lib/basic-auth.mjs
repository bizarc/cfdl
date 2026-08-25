/**
 * Credential checking for HTTP Basic authentication.
 *
 * Plain JavaScript rather than TypeScript so `scripts/check-middleware.mjs` can
 * import and exercise it with bare Node, no bundler and no test runner. The
 * middleware is a thin wrapper over these two functions; everything worth
 * getting wrong lives here.
 */

/**
 * Length-independent comparison, so a wrong guess cannot be narrowed by timing.
 * Both operands are short and already in memory, so the cost is irrelevant.
 *
 * @param {string} a
 * @param {string} b
 */
export function safeEqual(a, b) {
  if (typeof a !== "string" || typeof b !== "string") return false;
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

/**
 * Decide whether an Authorization header carries the expected credentials.
 *
 * Returns false for every malformed input rather than throwing: a request is
 * attacker-controlled, and the only two answers a caller wants are "let it
 * through" and "challenge it".
 *
 * A password may itself contain a colon, so the field separator is the FIRST
 * colon and everything after it is the password. Splitting on every colon
 * silently truncates such a password and rejects a correct credential.
 *
 * @param {string | null | undefined} header  the raw Authorization header
 * @param {string | undefined} user           expected username
 * @param {string | undefined} password       expected password
 */
export function isAuthorized(header, user, password) {
  // A deployment without configured credentials authorizes nobody. The caller
  // is responsible for turning that into a 503 rather than a challenge.
  if (!user || !password) return false;
  if (typeof header !== "string" || header.length === 0) return false;

  const space = header.indexOf(" ");
  if (space === -1) return false;

  const scheme = header.slice(0, space);
  const encoded = header.slice(space + 1).trim();
  // RFC 7235 makes the scheme case-insensitive.
  if (scheme.toLowerCase() !== "basic" || encoded.length === 0) return false;

  let decoded;
  try {
    decoded = atob(encoded);
  } catch {
    return false;
  }

  const separator = decoded.indexOf(":");
  if (separator === -1) return false;

  return (
    safeEqual(decoded.slice(0, separator), user) &&
    safeEqual(decoded.slice(separator + 1), password)
  );
}
