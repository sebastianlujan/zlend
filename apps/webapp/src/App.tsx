import { WagmiProvider } from "wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { config } from "./config/wagmi.ts";
import { addresses } from "./config/contracts.ts";
import { Header } from "./components/layout/Header.tsx";
import { AccountInfo } from "./components/wallet/AccountInfo.tsx";
import { TokenBalance } from "./components/dashboard/TokenBalance.tsx";
import { useAccount } from "wagmi";

const queryClient = new QueryClient();

function Dashboard() {
  const { isConnected } = useAccount();

  return (
    <main className="mx-auto max-w-5xl px-6 py-12">
      {isConnected && (
        <div className="mb-8">
          <AccountInfo />
        </div>
      )}

      <h2 className="mb-6 text-lg font-semibold text-surface-200">
        Token Balances
      </h2>

      <div className="grid gap-4 sm:grid-cols-2">
        <TokenBalance
          label="Collateral"
          tokenAddress={addresses.collateralToken}
        />
        <TokenBalance label="Borrow" tokenAddress={addresses.borrowToken} />
      </div>

      {!isConnected && (
        <p className="mt-12 text-center text-surface-500">
          Connect your wallet to view balances
        </p>
      )}
    </main>
  );
}

export default function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <Header />
        <Dashboard />
      </QueryClientProvider>
    </WagmiProvider>
  );
}
