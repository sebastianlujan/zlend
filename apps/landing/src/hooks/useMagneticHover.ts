import { useEffect, useRef, type RefObject } from "react";
import { gsap } from "../lib/gsap";
import { useReducedMotion } from "./useReducedMotion";

interface MagneticOptions {
  strength?: number;
}

export function useMagneticHover<T extends HTMLElement>(
  options: MagneticOptions = {},
): RefObject<T | null> {
  const { strength = 8 } = options;
  const ref = useRef<T | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = ref.current;
    if (!el || reduced) return;

    function onMove(e: MouseEvent) {
      const rect = el!.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      const deltaX = ((e.clientX - centerX) / (rect.width / 2)) * strength;
      const deltaY = ((e.clientY - centerY) / (rect.height / 2)) * strength;

      gsap.to(el, {
        x: deltaX,
        y: deltaY,
        duration: 0.3,
        ease: "power2.out",
      });
    }

    function onLeave() {
      gsap.to(el, {
        x: 0,
        y: 0,
        duration: 0.6,
        ease: "elastic.out(1, 0.5)",
      });
    }

    el.addEventListener("mousemove", onMove);
    el.addEventListener("mouseleave", onLeave);

    return () => {
      el.removeEventListener("mousemove", onMove);
      el.removeEventListener("mouseleave", onLeave);
      gsap.set(el, { x: 0, y: 0 });
    };
  }, [strength, reduced]);

  return ref;
}
