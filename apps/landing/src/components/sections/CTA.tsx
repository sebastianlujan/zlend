import { content } from "../../data/content";
import { Button } from "../ui/Button";
import { SectionWrapper } from "../ui/SectionWrapper";
import { useScrollAnimation } from "../../hooks/useScrollAnimation";

export function CTA() {
  const { cta } = content;
  const ref = useScrollAnimation<HTMLDivElement>({ childSelector: "[data-animate]" });

  return (
    <SectionWrapper id="cta">
      <div ref={ref} className="text-center">
        <h2
          data-animate
          className="text-4xl md:text-5xl font-bold text-white"
        >
          {cta.title}
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
      </div>
    </SectionWrapper>
  );
}
