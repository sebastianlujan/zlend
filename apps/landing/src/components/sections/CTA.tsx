import { useRef, useEffect } from "react";
import { content } from "../../data/content";
import { Button } from "../ui/Button";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";
import { animateTypewriter } from "../../lib/animations";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function CTA() {
  const { cta } = content;
  const ref = useScrollAnimation<HTMLDivElement>({
    childSelector: "[data-animate]",
    animation: "fadeUp",
  });
  const headingRef = useRef<HTMLHeadingElement>(null);
  const reduced = useReducedMotion();
  const hasTyped = useRef(false);

  useEffect(() => {
    const el = headingRef.current;
    if (!el || reduced || hasTyped.current) return;

    el.textContent = "";

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting || hasTyped.current) return;
        hasTyped.current = true;
        animateTypewriter(el, cta.title);
        observer.unobserve(el);
      },
      { threshold: 0.3 },
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [reduced, cta.title]);

  return (
    <SectionWrapper id="cta">
      <div ref={ref} className="text-center">
        <h2
          ref={headingRef}
          data-animate
          className="section-heading text-4xl md:text-5xl font-bold text-white font-mono cursor-blink"
        >
          {reduced ? cta.title : ""}
        </h2>
        <p data-animate className="mt-4 text-lg text-surface-400 max-w-xl mx-auto">
          {cta.subtitle}
        </p>
        <div data-animate className="mt-10 flex items-center justify-center gap-4 flex-wrap">
          {cta.links.map((link) => (
            <Button
              key={link.label}
              variant={link.icon === "docs" ? "primary" : "secondary"}
              size="lg"
              href={link.href}
            >
              {link.label}
            </Button>
          ))}
        </div>
        <div data-animate className="mt-8">
          <code className="text-sm font-mono text-surface-500">
            {cta.terminalCommand}
          </code>
        </div>
      </div>
    </SectionWrapper>
  );
}
