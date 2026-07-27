import type { Metadata } from "next";
import { SiteHeader } from "@/components/SiteHeader";
import { Playground } from "@/components/playground/Playground";

export const metadata: Metadata = {
  title: "Playground",
  description:
    "Compile and run CFDL models in your browser — the real compiler and engine, no install.",
};

export default function PlaygroundPage() {
  return (
    <>
      <SiteHeader />
      <Playground />
    </>
  );
}
