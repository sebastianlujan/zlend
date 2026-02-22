import { Header } from "./components/layout/Header";
import { Footer } from "./components/layout/Footer";
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
      <Header />
      <main>
        <Hero />
        <Problem />
        <Solution />
        <HowItWorks />
        <Features />
        <MarketData />
        <Trust />
        <Team />
        <CTA />
      </main>
      <Footer />
    </>
  );
}
