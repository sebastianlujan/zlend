import { useRef, useEffect } from "react";
import { content } from "../../data/content";
import { Button } from "../ui/Button";
import { Scene } from "../three/Scene";
import { gsap, scheduleAnimation } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function Hero() {
  const { hero } = content;
  const containerRef = useRef<HTMLDivElement>(null);
  const headlineRef = useRef<HTMLHeadingElement>(null);
  const subtitleRef = useRef<HTMLParagraphElement>(null);
  const teaserRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    if (!containerRef.current) return;

    const h1 = headlineRef.current;
    const subtitle = subtitleRef.current;
    const ctas = containerRef.current.querySelector("[data-ctas]");

    const teaser = teaserRef.current;

    if (reduced) {
      if (h1) h1.style.opacity = "1";
      if (subtitle) subtitle.style.opacity = "1";
      if (ctas instanceof HTMLElement) ctas.style.opacity = "1";
      if (teaser) teaser.style.opacity = "1";
      return;
    }

    if (h1) gsap.set(h1, { opacity: 0, y: 20 });
    if (subtitle) gsap.set(subtitle, { opacity: 0 });
    if (ctas instanceof HTMLElement) gsap.set(ctas, { opacity: 0 });
    if (teaser) gsap.set(teaser, { opacity: 0 });

    scheduleAnimation("critical", () => {
      const tl = gsap.timeline();

      if (h1) {
        tl.to(h1, { opacity: 1, y: 0, duration: 0.5, ease: "power3.out" });
      }

      if (subtitle) {
        tl.to(subtitle, { opacity: 1, duration: 0.5, ease: "power3.out" }, "-=0.2");
      }

      if (ctas instanceof HTMLElement) {
        tl.to(ctas, { opacity: 1, duration: 0.5, ease: "power3.out" }, "-=0.2");
      }

      if (teaser) {
        tl.to(teaser, { opacity: 1, y: 0, duration: 0.5, ease: "power3.out" }, "+=1.5");
      }
    });
  }, [reduced, hero.headline]);

  return (
    <section id="hero" className="scanlines relative min-h-screen flex items-center justify-center px-6 overflow-hidden">
      <Scene />
      <div
        ref={containerRef}
        className="relative z-10 text-center max-w-4xl mx-auto pt-20"
      >
        <h1
          ref={headlineRef}
          className="glow-text text-5xl md:text-7xl font-bold text-white leading-tight tracking-tight cursor-blink"
        >
          {hero.headline}
        </h1>
        <p
          ref={subtitleRef}
          className="mt-6 text-xl md:text-2xl text-surface-300 max-w-2xl mx-auto leading-relaxed font-mono"
        >
          {hero.subheadline}
        </p>
        <div data-ctas className="mt-10 flex items-center justify-center gap-4 flex-wrap">
          <Button variant="primary" size="lg" href={hero.cta.href}>
            {hero.cta.label}
          </Button>
          <Button variant="secondary" size="lg" href={hero.secondaryCta.href}>
            {hero.secondaryCta.label}
          </Button>
        </div>
      </div>

      {/* Scroll teaser */}
      <div
        ref={teaserRef}
        className="absolute bottom-8 left-1/2 -translate-x-1/2 z-10 flex flex-col items-center gap-2"
      >
        <span className="text-xs font-mono text-surface-500 cursor-blink">
          {">"} scroll to begin_
        </span>
        <svg
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          className="animate-bounce"
        >
          <path
            d="M3 5 L7 9 L11 5"
            stroke="rgba(156,156,166,0.4)"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </div>
    </section>
  );
}
