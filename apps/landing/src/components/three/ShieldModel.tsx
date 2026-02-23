import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import { Points, PointMaterial } from "@react-three/drei";
import type { Group, Points as PointsType } from "three";
import * as THREE from "three";

export function ShieldModel() {
  const groupRef = useRef<Group>(null);
  const orbitalRef = useRef<PointsType>(null);

  // Generate orbital particle positions in an elliptical orbit
  const orbitalPositions = useMemo(() => {
    const count = 50;
    const pos = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
      const angle = (i / count) * Math.PI * 2;
      pos[i * 3] = Math.cos(angle) * 2.8;
      pos[i * 3 + 1] = (Math.random() - 0.5) * 0.5;
      pos[i * 3 + 2] = Math.sin(angle) * 2.0;
    }
    return pos;
  }, []);

  useFrame((state, delta) => {
    const group = groupRef.current;
    if (!group) return;

    // Base rotation
    group.rotation.y += delta * 0.12;

    // Gentle bob
    group.position.y = Math.sin(state.clock.elapsedTime * 0.5) * 0.3;

    // Subtle mouse follow (max ~3deg)
    const targetX = state.pointer.y * 0.05;
    const targetY = state.pointer.x * 0.05;
    group.rotation.x = THREE.MathUtils.lerp(group.rotation.x, targetX, 0.05);
    group.children[0].rotation.z = THREE.MathUtils.lerp(
      group.children[0].rotation.z,
      targetY * 0.5,
      0.05,
    );

    // Orbital particles spin
    if (orbitalRef.current) {
      orbitalRef.current.rotation.y += delta * 0.3;
    }
  });

  return (
    <group ref={groupRef}>
      {/* Outer ring 1 */}
      <mesh>
        <torusGeometry args={[2.2, 0.06, 16, 64]} />
        <meshStandardMaterial
          color="#E84142"
          wireframe
          transparent
          opacity={0.2}
          emissive="#E84142"
          emissiveIntensity={0.1}
        />
      </mesh>

      {/* Outer ring 2 — perpendicular */}
      <mesh rotation={[Math.PI / 2, 0, 0]}>
        <torusGeometry args={[2.0, 0.06, 16, 64]} />
        <meshStandardMaterial
          color="#E84142"
          wireframe
          transparent
          opacity={0.15}
          emissive="#E84142"
          emissiveIntensity={0.08}
        />
      </mesh>

      {/* Inner shield — icosahedron */}
      <mesh>
        <icosahedronGeometry args={[1.2, 1]} />
        <meshStandardMaterial
          color="#058AFF"
          wireframe
          transparent
          opacity={0.12}
          emissive="#058AFF"
          emissiveIntensity={0.1}
        />
      </mesh>

      {/* Orbital particles */}
      <Points ref={orbitalRef} positions={orbitalPositions} stride={3} frustumCulled={false}>
        <PointMaterial
          transparent
          color="#ffffff"
          size={0.03}
          sizeAttenuation
          depthWrite={false}
          opacity={0.3}
        />
      </Points>
    </group>
  );
}
