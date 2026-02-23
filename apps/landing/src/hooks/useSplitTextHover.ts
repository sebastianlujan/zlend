import { useEffect, useRef, type RefObject } from "react";
import { gsap } from "../lib/gsap";
import { splitText } from "../lib/splitText";
import { useReducedMotion } from "./useReducedMotion";

export function useSplitTextHover<
  T extends HTMLElement,
>(): RefObject<T | null> {
  const ref = useRef<T | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = ref.current;
    if (!el || reduced) return;

    const { chars, revert } = splitText(el, "chars");
    if (chars.length === 0) {
      revert();
      return;
    }

    function onEnter() {
      gsap.to(chars, {
        y: -4,
        duration: 0.3,
        stagger: 0.02,
        ease: "power4.inOut",
      });
    }

    function onLeave() {
      gsap.to(chars, {
        y: 0,
        duration: 0.4,
        stagger: { each: 0.015, from: "random" },
        ease: "power2.out",
      });
    }

    el.addEventListener("mouseenter", onEnter);
    el.addEventListener("mouseleave", onLeave);

    return () => {
      el.removeEventListener("mouseenter", onEnter);
      el.removeEventListener("mouseleave", onLeave);
      revert();
    };
  }, [reduced]);

  return ref;
}
