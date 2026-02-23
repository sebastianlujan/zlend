import { Suspense, useMemo } from "react";
import { Canvas } from "@react-three/fiber";
import { ShieldModel } from "./ShieldModel";
import { ParticleField } from "./ParticleField";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function Scene() {
  const reduced = useReducedMotion();

  const isMobile = useMemo(
    () => typeof window !== "undefined" && window.innerWidth < 768,
    [],
  );

  if (reduced) return null;

  return (
    <div className="absolute inset-0 -z-10" aria-hidden="true">
      <Canvas
        camera={{ position: [0, 0, 8], fov: 60 }}
        dpr={isMobile ? [1, 1] : [1, 1.5]}
        gl={{ antialias: false, alpha: true }}
      >
        <ambientLight intensity={0.4} />
        <pointLight position={[10, 10, 10]} intensity={0.6} />
        <Suspense fallback={null}>
          <ShieldModel />
          <ParticleField count={isMobile ? 100 : 200} />
        </Suspense>
      </Canvas>
    </div>
  );
}
