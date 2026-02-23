import { useEffect, useRef, type RefObject } from "react";
import { gsap, ScrollTrigger } from "../lib/gsap";
import { useReducedMotion } from "./useReducedMotion";

interface ParallaxOptions {
  speed?: number;
  direction?: "vertical" | "horizontal";
  scale?: boolean;
}

export function useParallax<T extends HTMLElement>(
  options: ParallaxOptions = {},
): RefObject<T | null> {
  const { speed = 0.2, direction = "vertical", scale = false } = options;
  const ref = useRef<T | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = ref.current;
    if (!el || reduced) return;

    el.style.willChange = "transform";

    const yMove = direction === "vertical" ? speed * 100 : 0;
    const xMove = direction === "horizontal" ? speed * 100 : 0;

    const tween = gsap.to(el, {
      y: yMove,
      x: xMove,
      scale: scale ? 1 + Math.abs(speed) * 0.2 : 1,
      ease: "none",
      scrollTrigger: {
        trigger: el,
        start: "top bottom",
        end: "bottom top",
        scrub: 1,
      },
    });

    return () => {
      tween.kill();
      ScrollTrigger.getAll()
        .filter((st) => st.trigger === el)
        .forEach((st) => st.kill());
      el.style.willChange = "";
    };
  }, [speed, direction, scale, reduced]);

  return ref;
}
