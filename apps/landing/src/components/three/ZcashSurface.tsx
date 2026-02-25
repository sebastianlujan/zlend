import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";

import * as THREE from "three";
import {
  DEFAULT_DATA,
  type ZcashNetworkData,
} from "../../lib/zcashData";

// ── Vertex shader: flat grid displaced upward by Zcash data ──
const VERTEX_SHADER = /* glsl */ `
  uniform float uTime;
  uniform float uAmplitudes[20];
  uniform float uMempoolFactor;
  uniform float uDifficultyFactor;

  varying float vHeight;
  varying vec2 vUv;
  varying vec3 vWorldPos;

  void main() {
    vec2 uv = uv;
    float t = uTime;

    float x = uv.x * 6.28318;
    float z = uv.y * 6.28318;

    // ── Terrain displacement from Zcash block data ──
    float h = 0.0;

    // Base terrain undulation
    h += sin(x * 1.2 + t * 0.15) * cos(z * 0.9 + t * 0.12) * 0.6;
    h += sin(x * 0.7 - z * 0.5 + t * 0.08) * 0.4;

    // Block data — each of 20 recent blocks contributes a ridge
    for (int i = 0; i < 20; i++) {
      float amp = uAmplitudes[i] * 0.25;
      float freq = 1.0 + float(i) * 0.3;
      float phase = float(i) * 0.628;
      h += amp * sin(x * freq + t * 0.06 + phase)
               * cos(z * freq * 0.7 + t * 0.05);
    }
    h /= 3.0;

    // Difficulty — adds fine detail ridges
    h += sin(x * uDifficultyFactor * 3.0 + t * 0.04)
       * cos(z * uDifficultyFactor * 2.5 + t * 0.03) * 0.12;

    // Mempool — local turbulence hotspots
    h += sin(x * 6.0 + t * 0.7) * sin(z * 6.0 + t * 0.6)
       * uMempoolFactor * 0.07;

    // Displace Y
    vec3 pos = position;
    pos.y += h;

    vHeight = h;
    vUv = uv;
    vWorldPos = (modelMatrix * vec4(pos, 1.0)).xyz;

    gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
  }
`;

// ── Fragment shader: thermal colormap + grid + contours ──
const FRAGMENT_SHADER = /* glsl */ `
  uniform float uShieldedRatio;
  uniform float uTime;

  varying float vHeight;
  varying vec2 vUv;
  varying vec3 vWorldPos;

  // Thermal colormap: dark blue → blue → cyan → green → yellow → red → white
  vec3 thermalColor(float t) {
    // 7-stop gradient
    vec3 c0 = vec3(0.0, 0.0, 0.15);   // deep navy
    vec3 c1 = vec3(0.0, 0.1, 0.5);    // dark blue
    vec3 c2 = vec3(0.0, 0.4, 0.8);    // blue
    vec3 c3 = vec3(0.0, 0.8, 0.6);    // cyan-teal
    vec3 c4 = vec3(0.2, 0.9, 0.1);    // green
    vec3 c5 = vec3(0.9, 0.85, 0.0);   // yellow
    vec3 c6 = vec3(0.95, 0.2, 0.05);  // red
    vec3 c7 = vec3(1.0, 0.95, 0.9);   // hot white

    float s = clamp(t, 0.0, 1.0) * 7.0;
    if (s < 1.0) return mix(c0, c1, s);
    if (s < 2.0) return mix(c1, c2, s - 1.0);
    if (s < 3.0) return mix(c2, c3, s - 2.0);
    if (s < 4.0) return mix(c3, c4, s - 3.0);
    if (s < 5.0) return mix(c4, c5, s - 4.0);
    if (s < 6.0) return mix(c5, c6, s - 5.0);
    return mix(c6, c7, s - 6.0);
  }

  void main() {
    // Normalize height to 0..1 range for thermal map
    float hNorm = smoothstep(-0.35, 0.45, vHeight);

    // Shielded ratio shifts the color bias cooler
    float biased = mix(hNorm, hNorm * 0.6, uShieldedRatio * 0.5);
    vec3 color = thermalColor(biased);

    // ── Grid lines ──
    float gridSize = 320.0;
    float lineX = abs(fract(vUv.x * gridSize) - 0.5);
    float lineZ = abs(fract(vUv.y * gridSize) - 0.5);
    float grid = 1.0 - step(0.004, min(lineX, lineZ));
    vec3 gridColor = color * 1.3 + vec3(0.1);
    color = mix(color, gridColor, grid * 0.25);

    // ── Contour lines at height intervals (dense for rugosity) ──
    float contourInterval = 0.02;
    float contour = abs(fract(vHeight / contourInterval) - 0.5);
    float contourLine = 1.0 - smoothstep(0.003, 0.01, contour);
    color = mix(color, vec3(1.0), contourLine * 0.18);

    // Secondary coarser contours for major elevation bands
    float majorContour = abs(fract(vHeight / 0.08) - 0.5);
    float majorLine = 1.0 - smoothstep(0.004, 0.015, majorContour);
    color = mix(color, vec3(1.0), majorLine * 0.3);

    // ── Flat shading for polygon facets ──
    vec3 dx = dFdx(vWorldPos);
    vec3 dy = dFdy(vWorldPos);
    vec3 faceNormal = normalize(cross(dx, dy));
    vec3 lightDir = normalize(vec3(0.3, 0.8, 0.4));
    float diffuse = max(dot(faceNormal, lightDir), 0.0) * 0.5 + 0.5;
    color *= diffuse;

    // ── Edge fade ──
    float fadeX = smoothstep(0.0, 0.06, vUv.x) * smoothstep(0.0, 0.06, 1.0 - vUv.x);
    float fadeZ = smoothstep(0.0, 0.06, vUv.y) * smoothstep(0.0, 0.06, 1.0 - vUv.y);
    float fade = fadeX * fadeZ;

    float alpha = (0.6 + grid * 0.25 + contourLine * 0.08) * fade;

    gl_FragColor = vec4(color, alpha);
  }
`;

interface Props {
  isMobile: boolean;
  data: ZcashNetworkData;
}

const LERP_SPEED = 0.02;

export function ZcashSurface({ isMobile, data }: Props) {
  const groupRef = useRef<THREE.Group>(null);
  const materialRef = useRef<THREE.ShaderMaterial>(null);
  const currentData = useRef<ZcashNetworkData>(DEFAULT_DATA);

  const segments = isMobile ? 128 : 256;

  const geometry = useMemo(() => {
    const geo = new THREE.PlaneGeometry(8, 8, segments, segments);
    geo.rotateX(-Math.PI / 2); // lay flat on XZ plane
    return geo;
  }, [segments]);

  const initialUniforms = useMemo(
    () => ({
      uTime: { value: 0 },
      uAmplitudes: {
        value: DEFAULT_DATA.recentBlocks.map((b) => b.txCount / 10),
      },
      uMempoolFactor: { value: DEFAULT_DATA.mempoolSize / 50 },
      uDifficultyFactor: { value: 1.5 },
      uShieldedRatio: { value: DEFAULT_DATA.shieldedRatio },
    }),
    [],
  );

  useFrame((state, delta) => {
    const group = groupRef.current;
    const mat = materialRef.current;
    if (!group || !mat) return;

    const u = mat.uniforms;
    u.uTime.value = state.clock.elapsedTime;

    const target = data;
    const current = currentData.current;
    const lerpFactor = Math.min(LERP_SPEED * delta * 60, 1);

    current.shieldedRatio = THREE.MathUtils.lerp(
      current.shieldedRatio,
      target.shieldedRatio,
      lerpFactor,
    );
    u.uShieldedRatio.value = current.shieldedRatio;

    current.mempoolSize = THREE.MathUtils.lerp(
      current.mempoolSize,
      target.mempoolSize,
      lerpFactor,
    );
    u.uMempoolFactor.value = Math.min(current.mempoolSize / 50, 1);

    const targetDiffFactor =
      1.0 + Math.min(Math.log10(target.difficulty + 1) / 10, 2);
    u.uDifficultyFactor.value = THREE.MathUtils.lerp(
      u.uDifficultyFactor.value,
      targetDiffFactor,
      lerpFactor,
    );

    const amps = u.uAmplitudes.value as number[];
    for (let i = 0; i < 20; i++) {
      const targetAmp = (target.recentBlocks[i]?.txCount ?? 3) / 10;
      amps[i] = THREE.MathUtils.lerp(amps[i], targetAmp, lerpFactor);
    }

    // Slow Y rotation
    group.rotation.y += delta * 0.015;

    // Mouse tracking — subtle tilt
    const tiltX = state.pointer.y * 0.04;
    const tiltZ = state.pointer.x * 0.03;
    group.rotation.x = THREE.MathUtils.lerp(group.rotation.x, -0.15 + tiltX, 0.03);
    group.rotation.z = THREE.MathUtils.lerp(group.rotation.z, tiltZ, 0.03);
  });

  return (
    <group ref={groupRef} position={[0, -0.5, 0]}>
      <mesh geometry={geometry}>
        <shaderMaterial
          ref={materialRef}
          vertexShader={VERTEX_SHADER}
          fragmentShader={FRAGMENT_SHADER}
          uniforms={initialUniforms}
          transparent
          side={THREE.DoubleSide}
          depthWrite={false}
        />
      </mesh>
    </group>
  );
}
