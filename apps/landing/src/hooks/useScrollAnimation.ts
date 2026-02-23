import { useEffect, useRef, type RefObject } from "react";
import { gsap, ScrollTrigger, defaultScrollTrigger } from "../lib/gsap";
import { useReducedMotion } from "./useReducedMotion";

type AnimationType = "fadeUp" | "fadeIn";

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
  fadeUp: { from: { opacity: 0, y: 20 }, to: { opacity: 1, y: 0 } },
  fadeIn: { from: { opacity: 0 }, to: { opacity: 1 } },
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
      duration: 0.5,
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
