import { animate, stagger } from "animejs";

export function animateHeroEntrance(container: HTMLElement) {
  const children = container.querySelectorAll("[data-animate]");
  animate(children, {
    opacity: [0, 1],
    translateY: [40, 0],
    duration: 800,
    easing: "easeOutCubic",
    delay: stagger(120, { start: 200 }),
  });
}

export function animateCountUp(
  el: HTMLElement,
  target: number,
  duration = 1500,
) {
  animate(el, {
    innerHTML: [0, target],
    duration,
    round: 1,
    easing: "easeOutExpo",
  });
}
