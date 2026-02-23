import { MarketData } from "../components/sections/MarketData";
import { SectionDivider } from "../components/decorative/SectionDivider";
import { Trust } from "../components/sections/Trust";
import { CTA } from "../components/sections/CTA";

export function MarketPage() {
  return (
    <>
      <MarketData />
      <SectionDivider />
      <Trust />
      <SectionDivider />
      <CTA />
    </>
  );
}
