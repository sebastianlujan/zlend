import { useEffect, useRef, useState } from "react";
import { useLocation } from "react-router";
import { gsap, ScrollTrigger } from "../../lib/gsap";
import { useReducedMotion } from "../../hooks/useReducedMotion";
import { content } from "../../data/content";

export function ScrollProgress() {
  const barRef = useRef<HTMLDivElement>(null);
  const reduced = useReducedMotion();
  const [activeIndex, setActiveIndex] = useState(0);
  const { pathname } = useLocation();
  const isHome = pathname === "/";

  // Progress bar scrub
  useEffect(() => {
    const bar = barRef.current;
    if (!bar || reduced) return;

    gsap.to(bar, {
      scaleX: 1,
      ease: "none",
      scrollTrigger: {
        trigger: document.documentElement,
        start: "top top",
        end: "bottom bottom",
        scrub: 0.3,
      },
    });

    return () => {
      ScrollTrigger.getAll()
        .filter((st) => st.vars.trigger === document.documentElement)
        .forEach((st) => st.kill());
    };
  }, [reduced]);

  // Chapter dot tracking — only on home page
  useEffect(() => {
    if (reduced || !isHome) return;

    const triggers: ScrollTrigger[] = [];

    // Small delay to let sections mount
    const timer = setTimeout(() => {
      content.chapters.forEach((chapter, i) => {
        const el = document.getElementById(chapter.sectionId);
        if (!el) return;

        const st = ScrollTrigger.create({
          trigger: el,
          start: "top 60%",
          end: "bottom 40%",
          onEnter: () => setActiveIndex(i),
          onEnterBack: () => setActiveIndex(i),
        });

        triggers.push(st);
      });
    }, 100);

    return () => {
      clearTimeout(timer);
      triggers.forEach((st) => st.kill());
    };
  }, [reduced, isHome]);

  if (reduced) return null;

  return (
    <>
      {/* Progress bar */}
      <div
        ref={barRef}
        className="fixed top-0 left-0 right-0 h-0.5 bg-primary-500 z-[60] origin-left"
        style={{ transform: "scaleX(0)" }}
        aria-hidden="true"
      />

      {/* Chapter dots — desktop, home page only */}
      {isHome && (
        <nav
          className="fixed right-4 top-1/2 -translate-y-1/2 z-[60] hidden lg:flex flex-col items-end gap-3"
          aria-label="Page sections"
        >
          {content.chapters.map((chapter, i) => (
            <a
              key={chapter.sectionId}
              href={`#${chapter.sectionId}`}
              className="group flex items-center gap-2"
              onClick={(e) => {
                e.preventDefault();
                document.getElementById(chapter.sectionId)?.scrollIntoView({ behavior: "smooth" });
              }}
            >
              <span
                className={`text-[10px] font-mono uppercase tracking-wider transition-all duration-200 ${
                  i === activeIndex
                    ? "text-primary-400 opacity-100"
                    : "text-surface-600 opacity-0 group-hover:opacity-100"
                }`}
              >
                {chapter.label}
              </span>
              <span
                className={`block rounded-full transition-all duration-200 ${
                  i === activeIndex
                    ? "w-2 h-2 bg-primary-500"
                    : "w-1.5 h-1.5 bg-surface-600 group-hover:bg-surface-400"
                }`}
              />
            </a>
          ))}
        </nav>
      )}
    </>
  );
}
