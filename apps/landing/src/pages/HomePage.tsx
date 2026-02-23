import { content } from "../data/content";
import { Hero } from "../components/sections/Hero";
import { NarrativeBridge } from "../components/decorative/NarrativeBridge";
import { Problem } from "../components/sections/Problem";
import { MarketData } from "../components/sections/MarketData";
import { Solution } from "../components/sections/Solution";
import { HowItWorks } from "../components/sections/HowItWorks";
import { TrustCompact } from "../components/sections/TrustCompact";
import { CTA } from "../components/sections/CTA";

export function HomePage() {
  const bridges = content.narrativeBridges;

  return (
    <>
      <Hero />
      <NarrativeBridge prompt={bridges[0].prompt} />
      <Problem />
      <MarketData />
      <NarrativeBridge prompt={bridges[1].prompt} />
      <Solution />
      <HowItWorks />
      <NarrativeBridge prompt={bridges[2].prompt} />
      <TrustCompact />
      <CTA />
    </>
  );
}
