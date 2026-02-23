import { useParallax } from "../../hooks/useParallax";
import { useReducedMotion } from "../../hooks/useReducedMotion";

interface FloatingShape {
  type: "circle" | "hexagon" | "plus";
  x: string;
  y: string;
  size: number;
  color: string;
  opacity: number;
  speed: number;
}

const shapes: FloatingShape[] = [
  { type: "circle", x: "10%", y: "15%", size: 20, color: "#E84142", opacity: 0.06, speed: -0.3 },
  { type: "hexagon", x: "85%", y: "25%", size: 24, color: "#058AFF", opacity: 0.08, speed: 0.2 },
  { type: "plus", x: "75%", y: "55%", size: 16, color: "#E84142", opacity: 0.05, speed: -0.15 },
  { type: "circle", x: "20%", y: "70%", size: 14, color: "#058AFF", opacity: 0.07, speed: 0.4 },
  { type: "hexagon", x: "50%", y: "40%", size: 18, color: "#E84142", opacity: 0.05, speed: 0.5 },
  { type: "plus", x: "90%", y: "80%", size: 12, color: "#058AFF", opacity: 0.06, speed: -0.25 },
  { type: "circle", x: "5%", y: "90%", size: 22, color: "#E84142", opacity: 0.08, speed: 0.35 },
  { type: "hexagon", x: "60%", y: "10%", size: 16, color: "#058AFF", opacity: 0.12, speed: -0.2 },
];

function ShapeSVG({ type, size, color }: Pick<FloatingShape, "type" | "size" | "color">) {
  switch (type) {
    case "circle":
      return (
        <svg width={size} height={size} viewBox="0 0 24 24">
          <circle cx="12" cy="12" r="10" fill="none" stroke={color} strokeWidth="1" />
        </svg>
      );
    case "hexagon":
      return (
        <svg width={size} height={size} viewBox="0 0 24 24">
          <polygon points="12,2 22,8 22,16 12,22 2,16 2,8" fill="none" stroke={color} strokeWidth="1" />
        </svg>
      );
    case "plus":
      return (
        <svg width={size} height={size} viewBox="0 0 24 24">
          <line x1="12" y1="4" x2="12" y2="20" stroke={color} strokeWidth="1" />
          <line x1="4" y1="12" x2="20" y2="12" stroke={color} strokeWidth="1" />
        </svg>
      );
  }
}

function FloatingShape({ shape }: { shape: FloatingShape }) {
  const ref = useParallax<HTMLDivElement>({ speed: shape.speed });

  return (
    <div
      ref={ref}
      className="absolute"
      style={{
        left: shape.x,
        top: shape.y,
        opacity: shape.opacity,
      }}
    >
      <ShapeSVG type={shape.type} size={shape.size} color={shape.color} />
    </div>
  );
}

export function FloatingElements() {
  const reduced = useReducedMotion();
  if (reduced) return null;

  return (
    <div className="fixed inset-0 pointer-events-none z-0 overflow-hidden" aria-hidden="true">
      {shapes.map((shape, i) => (
        <FloatingShape key={i} shape={shape} />
      ))}
    </div>
  );
}
