import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import { Providers } from "@/components/Providers";
import { LearnHeader } from "@/components/LearnHeader";
import { LearnFooter } from "@/components/LearnFooter";
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

const SITE_URL = "https://learn.cfdl.dev";

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: {
    default: "CFDL Academy — learn cash-flow modeling in CFDL",
    template: "%s · CFDL Academy",
  },
  description:
    "A structured course in authoring cash-flow models with CFDL: the core language, modeling judgment, and a real-estate deal carried end to end — with runnable exercises in the browser.",
  openGraph: {
    type: "website",
    url: SITE_URL,
    siteName: "CFDL Academy",
    title: "CFDL Academy — learn cash-flow modeling in CFDL",
    description:
      "A structured course in authoring cash-flow models with CFDL, with runnable exercises in the browser.",
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
        <Providers>
          <LearnHeader />
          <main className="flex-1">{children}</main>
          <LearnFooter />
        </Providers>
      </body>
    </html>
  );
}
