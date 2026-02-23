import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

export { gsap, ScrollTrigger };

export const defaultScrollTrigger: ScrollTrigger.Vars = {
  start: "top 85%",
  toggleActions: "play none none none",
};

type Priority = "critical" | "normal" | "low";

export function scheduleAnimation(
  priority: Priority,
  callback: () => void,
): void {
  switch (priority) {
    case "critical":
      callback();
      break;
    case "normal":
      requestAnimationFrame(callback);
      break;
    case "low":
      if ("requestIdleCallback" in window) {
        window.requestIdleCallback(callback);
      } else {
        setTimeout(callback, 100);
      }
      break;
  }
}
