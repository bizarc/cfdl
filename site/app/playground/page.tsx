import type { Metadata } from "next";
import { SiteHeader } from "@/components/SiteHeader";
import { PlaygroundLoader } from "@/components/playground/PlaygroundLoader";

export const metadata: Metadata = {
  title: "Playground",
  description:
    "Compile and run CFDL models in your browser — the real compiler and engine, no install.",
};

export default function PlaygroundPage() {
  return (
    <>
      <SiteHeader />
      <PlaygroundLoader />
    </>
  );
}
