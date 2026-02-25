import { useState, useEffect, useRef } from "react";
import {
  fetchZcashData,
  DEFAULT_DATA,
  type ZcashNetworkData,
} from "../lib/zcashData";

const REFRESH_INTERVAL = 60_000;

export function useZcashData() {
  const [data, setData] = useState<ZcashNetworkData>(DEFAULT_DATA);
  const [live, setLive] = useState(false);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;

    const load = async () => {
      const result = await fetchZcashData();
      if (!mounted.current) return;
      setData(result);
      // If blockHeight differs from default, data is live
      if (result.blockHeight !== DEFAULT_DATA.blockHeight) setLive(true);
    };

    load();
    const interval = setInterval(load, REFRESH_INTERVAL);
    return () => {
      mounted.current = false;
      clearInterval(interval);
    };
  }, []);

  return { data, live };
}
