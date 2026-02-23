import { HowItWorks } from "../components/sections/HowItWorks";
import { SectionDivider } from "../components/decorative/SectionDivider";
import { Features } from "../components/sections/Features";
import { CTA } from "../components/sections/CTA";

export function TechnologyPage() {
  return (
    <>
      <HowItWorks />
      <SectionDivider />
      <Features />
      <SectionDivider />
      <CTA />
    </>
  );
}
