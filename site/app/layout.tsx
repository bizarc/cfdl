import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { Providers } from "@/components/Providers";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-jetbrains-mono",
  subsets: ["latin"],
  display: "swap",
});

const SITE_URL = "https://cfdl.dev";

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: "CFDL — the cash-flow modeling language",
    template: "%s · CFDL",
  },
  description:
    "A deterministic language for cash-flow models across energy, real estate, credit, and operating businesses. The same model file gives you the number and the distribution around it.",
  openGraph: {
    type: "website",
    url: SITE_URL,
    siteName: "CFDL",
    title: "CFDL — the cash-flow modeling language",
    description:
      "Deterministic, natively stochastic cash-flow models. Run them in your browser, your terminal, or your notebook.",
    images: ["/og.png"],
  },
  twitter: {
    card: "summary_large_image",
    title: "CFDL — the cash-flow modeling language",
    description:
      "Deterministic, natively stochastic cash-flow models. Run them in your browser, your terminal, or your notebook.",
    images: ["/og.png"],
  },
  icons: { icon: "/favicon.svg" },
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={`${inter.variable} ${jetbrainsMono.variable} h-full antialiased`}
    >
      <body className="flex min-h-full flex-col">
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
