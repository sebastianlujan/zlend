import { useState, useEffect, useRef } from "react";

const COINGECKO_URL =
  "https://api.coingecko.com/api/v3/simple/price?ids=zcash&vs_currencies=usd";
const REFRESH_INTERVAL = 60_000;

export function useZecPrice() {
  const [price, setPrice] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const cachedPrice = useRef<number | null>(null);

  useEffect(() => {
    let cancelled = false;

    const fetchPrice = async () => {
      try {
        const res = await fetch(COINGECKO_URL);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        const usd = data?.zcash?.usd;
        if (typeof usd !== "number") throw new Error("Invalid response");
        if (!cancelled) {
          cachedPrice.current = usd;
          setPrice(usd);
          setError(null);
          setLoading(false);
        }
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : "Failed to fetch price");
          if (cachedPrice.current) setPrice(cachedPrice.current);
          setLoading(false);
        }
      }
    };

    fetchPrice();
    const id = setInterval(fetchPrice, REFRESH_INTERVAL);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  return { price, loading, error };
}
