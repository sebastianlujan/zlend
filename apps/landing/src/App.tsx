import { Header } from "./components/layout/Header";
import { Footer } from "./components/layout/Footer";
import { ScrollProgress } from "./components/layout/ScrollProgress";
import { CustomCursor } from "./components/ui/CustomCursor";
import { FloatingElements } from "./components/decorative/FloatingElements";
import { SectionDivider } from "./components/decorative/SectionDivider";
import { Hero } from "./components/sections/Hero";
import { Problem } from "./components/sections/Problem";
import { Solution } from "./components/sections/Solution";
import { HowItWorks } from "./components/sections/HowItWorks";
import { Features } from "./components/sections/Features";
import { MarketData } from "./components/sections/MarketData";
import { Trust } from "./components/sections/Trust";
import { Team } from "./components/sections/Team";
import { CTA } from "./components/sections/CTA";

export function App() {
  return (
    <>
      <CustomCursor />
      <ScrollProgress />
      <FloatingElements />
      <Header />
      <main>
        <Hero />
        <SectionDivider />
        <Problem />
        <SectionDivider />
        <Solution />
        <SectionDivider />
        <HowItWorks />
        <SectionDivider />
        <Features />
        <SectionDivider />
        <MarketData />
        <SectionDivider />
        <Trust />
        <SectionDivider />
        <Team />
        <SectionDivider />
        <CTA />
      </main>
      <Footer />
    </>
  );
}
