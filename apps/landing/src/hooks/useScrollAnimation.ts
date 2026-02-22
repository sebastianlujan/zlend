import { useEffect, useRef, type RefObject } from "react";
import { animate, stagger } from "animejs";
import { useReducedMotion } from "./useReducedMotion";

interface ScrollAnimationOptions {
  threshold?: number;
  staggerDelay?: number;
  childSelector?: string;
}

export function useScrollAnimation<T extends HTMLElement>(
  options: ScrollAnimationOptions = {},
): RefObject<T | null> {
  const { threshold = 0.15, staggerDelay = 80, childSelector } = options;
  const ref = useRef<T | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = ref.current;
    if (!el || reduced) return;

    el.style.opacity = "0";

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting) return;

        const targets = childSelector
          ? el.querySelectorAll(childSelector)
          : el;

        animate(targets, {
          opacity: [0, 1],
          translateY: [30, 0],
          duration: 600,
          easing: "easeOutCubic",
          delay: childSelector ? stagger(staggerDelay) : 0,
        });

        el.style.opacity = "1";
        observer.unobserve(el);
      },
      { threshold },
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [threshold, staggerDelay, childSelector, reduced]);

  return ref;
}
