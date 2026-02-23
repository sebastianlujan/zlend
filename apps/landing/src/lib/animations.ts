import { gsap } from "./gsap";

export function animateHeroEntrance(container: HTMLElement) {
  const children = container.querySelectorAll("[data-animate]");
  gsap.fromTo(
    children,
    { opacity: 0, y: 40 },
    {
      opacity: 1,
      y: 0,
      duration: 0.8,
      ease: "power3.out",
      stagger: 0.12,
      delay: 0.2,
    },
  );
}

export function animateTypewriter(
  el: HTMLElement,
  text: string,
  onComplete?: () => void,
) {
  el.textContent = "";
  el.style.opacity = "1";

  const progress = { value: 0 };
  const totalChars = text.length;
  const duration = totalChars * 0.05;

  const tween = gsap.to(progress, {
    value: totalChars,
    duration,
    ease: "none",
    onUpdate() {
      el.textContent = text.slice(0, Math.round(progress.value));
    },
    onComplete() {
      onComplete?.();
    },
  });

  return {
    duration: duration * 1000,
    cancel() {
      tween.kill();
    },
  };
}

export function animateCountUp(
  el: HTMLElement,
  target: number,
  duration = 1.5,
) {
  const decimals = (target.toString().split(".")[1] ?? "").length;
  const obj = { value: 0 };

  gsap.to(obj, {
    value: target,
    duration,
    ease: "power2.out",
    onUpdate() {
      el.textContent = obj.value.toFixed(decimals);
    },
  });
}
