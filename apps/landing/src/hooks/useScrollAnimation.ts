import { useEffect, useRef, type RefObject } from "react";
import { gsap, ScrollTrigger, defaultScrollTrigger } from "../lib/gsap";
import { useReducedMotion } from "./useReducedMotion";

type AnimationType =
  | "fadeUp"
  | "fadeIn"
  | "scaleUp"
  | "slideLeft"
  | "slideRight";

interface ScrollAnimationOptions {
  threshold?: number;
  staggerDelay?: number;
  childSelector?: string;
  animation?: AnimationType;
}

const animationPresets: Record<
  AnimationType,
  { from: gsap.TweenVars; to: gsap.TweenVars }
> = {
  fadeUp: { from: { opacity: 0, y: 30 }, to: { opacity: 1, y: 0 } },
  fadeIn: { from: { opacity: 0 }, to: { opacity: 1 } },
  scaleUp: {
    from: { opacity: 0, scale: 0.85 },
    to: { opacity: 1, scale: 1 },
  },
  slideLeft: { from: { opacity: 0, x: -60 }, to: { opacity: 1, x: 0 } },
  slideRight: { from: { opacity: 0, x: 60 }, to: { opacity: 1, x: 0 } },
};

export function useScrollAnimation<T extends HTMLElement>(
  options: ScrollAnimationOptions = {},
): RefObject<T | null> {
  const {
    staggerDelay = 0.08,
    childSelector,
    animation = "fadeUp",
  } = options;
  const ref = useRef<T | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = ref.current;
    if (!el || reduced) return;

    const preset = animationPresets[animation];
    const targets = childSelector ? el.querySelectorAll(childSelector) : el;

    gsap.set(targets, preset.from);

    const tl = gsap.timeline({
      scrollTrigger: {
        trigger: el,
        ...defaultScrollTrigger,
      },
    });

    tl.to(targets, {
      ...preset.to,
      duration: 0.6,
      ease: "power3.out",
      stagger: childSelector ? staggerDelay : 0,
      onStart() {
        el.classList.add("in-view");
      },
    });

    return () => {
      tl.kill();
      ScrollTrigger.getAll()
        .filter((st) => st.trigger === el)
        .forEach((st) => st.kill());
    };
  }, [staggerDelay, childSelector, reduced, animation]);

  return ref;
}
