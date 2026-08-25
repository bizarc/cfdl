import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

import { isAuthorized } from "./lib/basic-auth.mjs";

/**
 * Basic authentication for everything under `/private`.
 *
 * The page this guards was previously shared as a static HTML file carrying its
 * own passphrase prompt. That prompt was never access control: the whole
 * document shipped inside the markup and view-source defeated it. Here the
 * check happens at the edge, so an unauthenticated request is answered with a
 * challenge and never receives the content at all.
 *
 * The credential comparison lives in `lib/basic-auth.mjs` so it can be
 * exercised by `scripts/check-middleware.mjs` without a bundler or a test
 * runner. See `npm run check:middleware`.
 */
export function middleware(request: NextRequest) {
  const user = process.env.PRIVATE_PAGE_USER;
  const password = process.env.PRIVATE_PAGE_PASSWORD;

  // A deployment that has not set the credentials serves 503 rather than
  // falling open. A missing secret must never be the same thing as a public
  // page — that failure mode is silent and indistinguishable from success.
  if (!user || !password) {
    return new NextResponse("Private pages are not configured on this deployment.", {
      status: 503,
      headers: { "cache-control": "no-store" },
    });
  }

  if (isAuthorized(request.headers.get("authorization"), user, password)) {
    const response = NextResponse.next();
    // Never let a shared cache hold a page that required credentials.
    response.headers.set("cache-control", "private, no-store");
    response.headers.set("x-robots-tag", "noindex, nofollow, noarchive");
    return response;
  }

  return new NextResponse("Authentication required.", {
    status: 401,
    headers: {
      "www-authenticate": 'Basic realm="CFDL private", charset="UTF-8"',
      "cache-control": "no-store",
    },
  });
}

export const config = {
  matcher: ["/private/:path*"],
};
