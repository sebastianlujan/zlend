import { type ZcashNetworkData } from "../../lib/zcashData";

interface Props {
  data: ZcashNetworkData;
  live: boolean;
}

function fmt(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(2) + "M";
  if (n >= 1_000) return (n / 1_000).toFixed(1) + "K";
  return n.toLocaleString();
}

export function ZcashHud({ data, live }: Props) {
  const avgTx =
    data.recentBlocks.length > 0
      ? (
          data.recentBlocks.reduce((s, b) => s + b.txCount, 0) /
          data.recentBlocks.length
        ).toFixed(1)
      : "—";

  return (
    <div className="absolute bottom-6 left-6 z-20 hidden md:block font-mono text-xs text-surface-400 bg-surface-950/70 border border-surface-800 rounded px-4 py-3 backdrop-blur-sm">
      <div className="flex items-center gap-2 mb-2 text-surface-300">
        <span
          className={`inline-block w-1.5 h-1.5 rounded-full ${live ? "bg-green-400" : "bg-surface-600"}`}
        />
        <span className="uppercase tracking-widest text-[10px]">
          Zcash Network
        </span>
      </div>
      <div className="space-y-1">
        <Row label="Block" value={`#${data.blockHeight.toLocaleString()}`} />
        <Row label="Orchard" value={`${fmt(data.shieldedPools.orchard)} ZEC`} />
        <Row label="Sapling" value={`${fmt(data.shieldedPools.sapling)} ZEC`} />
        <Row
          label="Shielded"
          value={`${(data.shieldedRatio * 100).toFixed(1)}%`}
        />
        <Row label="Difficulty" value={fmt(data.difficulty)} />
        <Row label="Avg tx/blk" value={avgTx} />
        <Row label="Mempool" value={String(data.mempoolSize)} />
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between gap-6">
      <span className="text-surface-500">{label}</span>
      <span className="text-surface-300">{value}</span>
    </div>
  );
}
