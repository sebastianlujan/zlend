import { useRef, useEffect } from "react";
import { content } from "../../data/content";
import { Button } from "../ui/Button";
import { Scene } from "../three/Scene";
import { animateHeroEntrance } from "../../lib/animations";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function Hero() {
  const { hero } = content;
  const containerRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    if (containerRef.current && !reduced) {
      animateHeroEntrance(containerRef.current);
    }
  }, [reduced]);

  return (
    <section className="relative min-h-screen flex items-center justify-center px-6 overflow-hidden">
      <Scene />
      <div
        ref={containerRef}
        className="relative z-10 text-center max-w-4xl mx-auto pt-20"
      >
        <h1
          data-animate
          className="text-5xl md:text-7xl font-bold text-white leading-tight tracking-tight"
        >
          {hero.headline}
        </h1>
        <p
          data-animate
          className="mt-6 text-xl md:text-2xl text-surface-300 max-w-2xl mx-auto leading-relaxed"
        >
          {hero.subheadline}
        </p>
        <div data-animate className="mt-10 flex items-center justify-center gap-4 flex-wrap">
          <Button variant="primary" size="lg" href={hero.cta.href}>
            {hero.cta.label}
          </Button>
          <Button variant="secondary" size="lg" href={hero.secondaryCta.href}>
            {hero.secondaryCta.label}
          </Button>
        </div>
      </div>

      <div className="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-surface-950 to-transparent" />
    </section>
  );
}
