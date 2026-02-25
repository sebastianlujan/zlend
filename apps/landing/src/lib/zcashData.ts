const TATUM_GATEWAY = "https://zcash-mainnet.gateway.tatum.io";
const API_KEY = import.meta.env.VITE_TATUM_API_KEY as string | undefined;
const ZEC_TOTAL_SUPPLY = 21_000_000;

export interface ZcashNetworkData {
  shieldedPools: { sprout: number; sapling: number; orchard: number };
  totalShielded: number;
  shieldedRatio: number;
  difficulty: number;
  blockHeight: number;
  recentBlocks: { txCount: number; fees: number }[];
  mempoolSize: number;
}

export const DEFAULT_DATA: ZcashNetworkData = {
  shieldedPools: { sprout: 25_000, sapling: 620_000, orchard: 4_390_000 },
  totalShielded: 5_035_000,
  shieldedRatio: 0.3,
  difficulty: 120_000_000,
  blockHeight: 3_250_000,
  recentBlocks: Array.from({ length: 20 }, (_, i) => ({
    txCount: 3 + Math.round(Math.sin(i * 0.7) * 2),
    fees: 0.0001,
  })),
  mempoolSize: 12,
};

let rpcId = 0;

async function rpcCall<T>(method: string, params: unknown[] = []): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (API_KEY) headers["x-api-key"] = API_KEY;

  const res = await fetch(TATUM_GATEWAY, {
    method: "POST",
    headers,
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: ++rpcId,
      method,
      params,
    }),
  });

  if (!res.ok) throw new Error(`RPC ${method} failed: ${res.status}`);
  const json = await res.json();
  if (json.error) throw new Error(`RPC ${method} error: ${json.error.message}`);
  return json.result as T;
}

interface BlockchainInfo {
  blocks: number;
  difficulty: number;
  valuePools?: {
    id: string;
    monitored: boolean;
    chainValue?: number;
    chainValueZat?: number;
  }[];
}

interface BlockData {
  tx: string[];
  difficulty: number;
  chainSupply?: {
    valueDelta?: number;
    valueDeltaZat?: number;
  };
}

interface MempoolInfo {
  size: number;
}

export async function fetchZcashData(): Promise<ZcashNetworkData> {
  try {
    // Step 1: blockchain info + mempool in parallel
    const [chainInfo, mempoolInfo] = await Promise.all([
      rpcCall<BlockchainInfo>("getblockchaininfo"),
      rpcCall<MempoolInfo>("getmempoolinfo"),
    ]);

    const blockHeight = chainInfo.blocks;

    // Step 2: get hashes for last 20 blocks
    const hashPromises: Promise<string>[] = [];
    for (let i = 0; i < 20; i++) {
      const height = blockHeight - i;
      if (height > 0) {
        hashPromises.push(rpcCall<string>("getblockhash", [height]));
      }
    }
    const hashes = await Promise.all(hashPromises);

    // Step 3: get block data (verbosity 1) for each hash
    const blockPromises = hashes.map((hash) =>
      rpcCall<BlockData>("getblock", [hash, 1]),
    );
    const blocks = await Promise.all(blockPromises);

    // Parse shielded pools
    const pools = chainInfo.valuePools ?? [];
    const sprout =
      pools.find((p) => p.id === "sprout")?.chainValue ??
      DEFAULT_DATA.shieldedPools.sprout;
    const sapling =
      pools.find((p) => p.id === "sapling")?.chainValue ??
      DEFAULT_DATA.shieldedPools.sapling;
    const orchard =
      pools.find((p) => p.id === "orchard")?.chainValue ??
      DEFAULT_DATA.shieldedPools.orchard;
    const totalShielded = sprout + sapling + orchard;

    const recentBlocks = blocks.map((block) => ({
      txCount: block.tx?.length ?? 1,
      fees: block.chainSupply?.valueDelta ?? 0.0001,
    }));

    return {
      shieldedPools: { sprout, sapling, orchard },
      totalShielded,
      shieldedRatio: Math.min(totalShielded / ZEC_TOTAL_SUPPLY, 1),
      difficulty: chainInfo.difficulty ?? DEFAULT_DATA.difficulty,
      blockHeight,
      recentBlocks,
      mempoolSize: mempoolInfo.size ?? DEFAULT_DATA.mempoolSize,
    };
  } catch (err) {
    console.warn("[zcashData] Fetch failed, using defaults:", err);
    return DEFAULT_DATA;
  }
}
