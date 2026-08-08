import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  env: {
    // "Open in playground" on a code block deep-links into the site's
    // playground; the model travels in the URL fragment.
    NEXT_PUBLIC_PLAYGROUND_ORIGIN: "https://cfdl.dev",
  },
};

export default nextConfig;
