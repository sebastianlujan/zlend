import { useEffect, useRef, type RefObject } from "react";
import { gsap } from "../lib/gsap";
import { useReducedMotion } from "./useReducedMotion";

interface TiltOptions {
  maxDeg?: number;
}

export function useTiltHover<T extends HTMLElement>(
  options: TiltOptions = {},
): RefObject<T | null> {
  const { maxDeg = 6 } = options;
  const ref = useRef<T | null>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const el = ref.current;
    if (!el || reduced) return;

    function onMove(e: MouseEvent) {
      const rect = el!.getBoundingClientRect();
      const x = (e.clientX - rect.left) / rect.width - 0.5;
      const y = (e.clientY - rect.top) / rect.height - 0.5;

      gsap.to(el, {
        rotateX: -y * maxDeg,
        rotateY: x * maxDeg,
        transformPerspective: 800,
        duration: 0.3,
        ease: "power2.out",
      });
    }

    function onLeave() {
      gsap.to(el, {
        rotateX: 0,
        rotateY: 0,
        duration: 0.5,
        ease: "power2.out",
      });
    }

    el.addEventListener("mousemove", onMove);
    el.addEventListener("mouseleave", onLeave);

    return () => {
      el.removeEventListener("mousemove", onMove);
      el.removeEventListener("mouseleave", onLeave);
      gsap.set(el, { rotateX: 0, rotateY: 0 });
    };
  }, [maxDeg, reduced]);

  return ref;
}
