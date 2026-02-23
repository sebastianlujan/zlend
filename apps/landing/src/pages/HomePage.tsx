import { Hero } from "../components/sections/Hero";
import { SectionDivider } from "../components/decorative/SectionDivider";
import { Problem } from "../components/sections/Problem";
import { Solution } from "../components/sections/Solution";
import { CTA } from "../components/sections/CTA";

export function HomePage() {
  return (
    <>
      <Hero />
      <SectionDivider />
      <Problem />
      <SectionDivider />
      <Solution />
      <SectionDivider />
      <CTA />
    </>
  );
}
