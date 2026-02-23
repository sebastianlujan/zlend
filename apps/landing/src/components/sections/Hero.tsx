import { useRef, useEffect } from "react";
import { content } from "../../data/content";
import { Button } from "../ui/Button";
import { Scene } from "../three/Scene";
import { gsap, scheduleAnimation } from "../../lib/gsap";
import { splitText } from "../../lib/splitText";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function Hero() {
  const { hero } = content;
  const containerRef = useRef<HTMLDivElement>(null);
  const headlineRef = useRef<HTMLHeadingElement>(null);
  const subtitleRef = useRef<HTMLParagraphElement>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    if (!containerRef.current) return;

    const h1 = headlineRef.current;
    const subtitle = subtitleRef.current;
    const ctas = containerRef.current.querySelector("[data-ctas]");

    if (reduced) {
      if (h1) h1.textContent = hero.headline;
      if (subtitle) subtitle.style.opacity = "1";
      if (ctas instanceof HTMLElement) ctas.style.opacity = "1";
      return;
    }

    // Set initial text content for splitting
    if (h1) h1.textContent = hero.headline;
    if (subtitle) subtitle.style.opacity = "0";
    if (ctas instanceof HTMLElement) ctas.style.opacity = "0";

    let revert: (() => void) | undefined;

    scheduleAnimation("critical", () => {
      if (!h1) return;

      const { chars, revert: revertFn } = splitText(h1, "chars");
      revert = revertFn;

      const tl = gsap.timeline();

      // Character-level entrance
      tl.fromTo(
        chars,
        { opacity: 0, y: "100%", rotateX: -90 },
        {
          opacity: 1,
          y: 0,
          rotateX: 0,
          duration: 0.8,
          stagger: 0.03,
          ease: "power4.out",
        },
      );

      // Subtitle fades in 0.3s after chars complete
      if (subtitle) {
        tl.to(
          subtitle,
          { opacity: 1, duration: 0.6, ease: "power2.out" },
          "-=0.3",
        );
      }

      // CTAs cascade in
      if (ctas instanceof HTMLElement) {
        tl.to(
          ctas,
          { opacity: 1, duration: 0.5, ease: "power2.out" },
          "-=0.45",
        );
      }
    });

    return () => {
      revert?.();
    };
  }, [reduced, hero.headline]);

  return (
    <section className="scanlines relative min-h-screen flex items-center justify-center px-6 overflow-hidden">
      <Scene />
      <div
        ref={containerRef}
        className="relative z-10 text-center max-w-4xl mx-auto pt-20"
      >
        <h1
          ref={headlineRef}
          aria-label={hero.headline}
          className="glow-text text-5xl md:text-7xl font-bold text-white leading-tight tracking-tight cursor-blink"
        />
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
    </section>
  );
}
