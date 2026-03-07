import { http, createConfig } from "wagmi";
import { avalanche, avalancheFuji } from "wagmi/chains";
import { injected, walletConnect } from "wagmi/connectors";

const projectId = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID as
  | string
  | undefined;

const connectors = [
  injected(),
  ...(projectId ? [walletConnect({ projectId })] : []),
];

export const config = createConfig({
  chains: [avalanche, avalancheFuji],
  connectors,
  transports: {
    [avalanche.id]: import.meta.env.VITE_RPC_URL
      ? http(import.meta.env.VITE_RPC_URL as string)
      : http(),
    [avalancheFuji.id]: http(),
  },
});

declare module "wagmi" {
  interface Register {
    config: typeof config;
  }
}
