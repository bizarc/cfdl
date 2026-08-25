import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

/**
 * Basic authentication for everything under `/private`.
 *
 * The page this guards was previously shared as a static HTML file carrying its
 * own passphrase prompt. That prompt was never access control: the whole
 * document shipped inside the markup and view-source defeated it. Here the
 * check happens at the edge, so an unauthenticated request is answered with a
 * challenge and never receives the content at all.
 *
 * Credentials come from the environment. A deployment that has not set them
 * serves 503 rather than falling open — a missing secret must never be the same
 * thing as a public page.
 */
export function middleware(request: NextRequest) {
  const user = process.env.PRIVATE_PAGE_USER;
  const password = process.env.PRIVATE_PAGE_PASSWORD;

  if (!user || !password) {
    return new NextResponse("Private pages are not configured on this deployment.", {
      status: 503,
      headers: { "cache-control": "no-store" },
    });
  }

  const header = request.headers.get("authorization") ?? "";
  const [scheme, encoded] = header.split(" ");

  if (scheme === "Basic" && encoded) {
    // atob is available in the edge runtime; Buffer is not.
    let decoded = "";
    try {
      decoded = atob(encoded);
    } catch {
      decoded = "";
    }
    const separator = decoded.indexOf(":");
    if (separator !== -1) {
      const candidateUser = decoded.slice(0, separator);
      const candidatePassword = decoded.slice(separator + 1);
      if (safeEqual(candidateUser, user) && safeEqual(candidatePassword, password)) {
        const response = NextResponse.next();
        // Never let a shared cache hold a page that required credentials.
        response.headers.set("cache-control", "private, no-store");
        response.headers.set("x-robots-tag", "noindex, nofollow, noarchive");
        return response;
      }
    }
  }

  return new NextResponse("Authentication required.", {
    status: 401,
    headers: {
      "www-authenticate": 'Basic realm="CFDL private", charset="UTF-8"',
      "cache-control": "no-store",
    },
  });
}

/**
 * Length-independent comparison, so a wrong guess cannot be narrowed by timing.
 * Both operands are short and already in memory, so the cost is irrelevant.
 */
function safeEqual(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

export const config = {
  matcher: ["/private/:path*"],
};
