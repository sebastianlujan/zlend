import { Suspense, useMemo } from "react";
import { Canvas } from "@react-three/fiber";
import { ZcashSurface } from "./ZcashSurface";

import { ParticleField } from "./ParticleField";
import { useReducedMotion } from "../../hooks/useReducedMotion";
import { type ZcashNetworkData } from "../../lib/zcashData";

interface Props {
  data: ZcashNetworkData;
}

export function Scene({ data }: Props) {
  const reduced = useReducedMotion();

  const isMobile = useMemo(
    () => typeof window !== "undefined" && window.innerWidth < 768,
    [],
  );

  if (reduced) return null;

  const frustum = isMobile ? 4 : 6;

  return (
    <div className="absolute inset-0 -z-10" aria-hidden="true">
      <Canvas
        orthographic
        camera={{
          position: [5, 6, 5],
          zoom: isMobile ? 22 : 32,
          near: -100,
          far: 200,
          left: -frustum,
          right: frustum,
          top: frustum,
          bottom: -frustum,
        }}
        dpr={isMobile ? [1, 1] : [1, 1.5]}
        gl={{ antialias: true, alpha: true }}
      >
        <ambientLight intensity={0.4} />
        <pointLight position={[10, 10, 10]} intensity={0.6} />
        <Suspense fallback={null}>
          <ZcashSurface isMobile={isMobile} data={data} />
          <ParticleField count={isMobile ? 100 : 200} />
        </Suspense>
      </Canvas>
    </div>
  );
}
