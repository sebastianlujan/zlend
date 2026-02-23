import { useEffect } from "react";
import { useLocation } from "react-router";
import { ScrollTrigger } from "../../lib/gsap";

export function ScrollToTop() {
  const { pathname } = useLocation();

  useEffect(() => {
    window.scrollTo(0, 0);
    requestAnimationFrame(() => {
      ScrollTrigger.refresh();
    });
  }, [pathname]);

  return null;
}
