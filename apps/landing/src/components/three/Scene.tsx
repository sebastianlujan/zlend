import { Suspense } from "react";
import { Canvas } from "@react-three/fiber";
import { ParticleField } from "./ParticleField";
import { ShieldModel } from "./ShieldModel";
import { useReducedMotion } from "../../hooks/useReducedMotion";

export function Scene() {
  const reduced = useReducedMotion();

  if (reduced) return null;

  return (
    <div className="absolute inset-0 -z-10" aria-hidden="true">
      <Canvas
        camera={{ position: [0, 0, 8], fov: 60 }}
        dpr={[1, 1.5]}
        gl={{ antialias: false, alpha: true }}
      >
        <ambientLight intensity={0.4} />
        <pointLight position={[10, 10, 10]} intensity={0.6} />
        <Suspense fallback={null}>
          <ParticleField />
          <ShieldModel />
        </Suspense>
      </Canvas>
    </div>
  );
}
