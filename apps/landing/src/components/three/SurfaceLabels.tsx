import { Html } from "@react-three/drei";
import { type ZcashNetworkData } from "../../lib/zcashData";

interface Props {
  data: ZcashNetworkData;
}

function fmt(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M";
  if (n >= 1_000) return (n / 1_000).toFixed(0) + "K";
  return n.toLocaleString();
}

const labelStyle: React.CSSProperties = {
  fontFamily: "monospace",
  fontSize: "9px",
  color: "rgba(255,255,255,0.45)",
  whiteSpace: "nowrap",
  pointerEvents: "none",
  userSelect: "none",
};

const accentStyle: React.CSSProperties = {
  ...labelStyle,
  color: "rgba(255,255,255,0.6)",
  fontSize: "10px",
};

export function SurfaceLabels({ data }: Props) {
  const halfSize = 4; // plane is 8x8, so half = 4

  // Pick 5 recent blocks for edge ticks
  const blockTicks = [0, 4, 9, 14, 19].map((i) => ({
    idx: i,
    tx: data.recentBlocks[i]?.txCount ?? 0,
    height: data.blockHeight - i,
  }));

  return (
    <group>
      {/* ── X axis: block heights along front edge ── */}
      {blockTicks.map((tick, i) => {
        const x = -halfSize + (i / (blockTicks.length - 1)) * halfSize * 2;
        return (
          <Html
            key={`x-${tick.idx}`}
            position={[x, -0.1, halfSize + 0.3]}
            center
            style={labelStyle}
          >
            {`#${fmt(tick.height)}`}
          </Html>
        );
      })}

      {/* ── Z axis: tx counts along right edge ── */}
      {blockTicks.map((tick, i) => {
        const z = -halfSize + (i / (blockTicks.length - 1)) * halfSize * 2;
        return (
          <Html
            key={`z-${tick.idx}`}
            position={[halfSize + 0.3, -0.1, z]}
            center
            style={labelStyle}
          >
            {`${tick.tx}tx`}
          </Html>
        );
      })}

      {/* ── Y axis: height markers on left edge ── */}
      {[-0.3, 0, 0.3, 0.6].map((y) => (
        <Html
          key={`y-${y}`}
          position={[-halfSize - 0.3, y, -halfSize]}
          center
          style={labelStyle}
        >
          {y.toFixed(1)}
        </Html>
      ))}

      {/* ── Corner data labels ── */}
      <Html position={[-halfSize, -0.3, halfSize + 0.6]} center style={accentStyle}>
        {`MEMPOOL: ${data.mempoolSize}`}
      </Html>
      <Html position={[halfSize, -0.3, -halfSize - 0.6]} center style={accentStyle}>
        {`SHIELDED: ${(data.shieldedRatio * 100).toFixed(1)}%`}
      </Html>
      <Html position={[-halfSize, -0.3, -halfSize - 0.6]} center style={accentStyle}>
        {`DIFF: ${fmt(data.difficulty)}`}
      </Html>
      <Html position={[halfSize, -0.3, halfSize + 0.6]} center style={accentStyle}>
        {`BLK: #${data.blockHeight.toLocaleString()}`}
      </Html>
    </group>
  );
}
