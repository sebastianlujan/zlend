import { WagmiProvider, useAccount, useConnect } from "wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Wallet } from "lucide-react";
import { config } from "./config/wagmi.ts";
import { Header } from "./components/layout/Header.tsx";
import { SummaryStrip } from "./components/dashboard/SummaryStrip.tsx";
import { DepositView } from "./components/views/DepositView.tsx";
import { useVaults } from "./hooks/useVaults.ts";
import { Button } from "./components/ui/Button.tsx";
import { ToastProvider } from "./components/ui/ToastContainer.tsx";

const queryClient = new QueryClient();

function Dashboard() {
  const { isConnected } = useAccount();
  const { connect, connectors } = useConnect();
  const { vaults, addVault, updateVault } = useVaults();

  if (!isConnected) {
    return (
      <>
        <Header />
        <main className="mx-auto max-w-5xl px-6 py-20 text-center">
          <h2 className="text-2xl font-bold text-white">
            Private lending, powered by ZK proofs
          </h2>
          <p className="mt-3 text-surface-400">
            Borrow USDT against your ZEC holdings — without revealing your
            identity.
          </p>
          <div className="mt-8">
            <Button size="lg" onClick={() => connect({ connector: connectors[0] })}>
              <Wallet className="mr-2 h-5 w-5" />
              Connect Wallet
            </Button>
          </div>
        </main>
      </>
    );
  }

  return (
    <>
      <Header />
      <main className="mx-auto max-w-5xl px-6 py-8">
        <div className="mb-8">
          <SummaryStrip vaults={vaults} />
        </div>

        <DepositView
          vaults={vaults}
          onAddVault={addVault}
          onUpdateVault={updateVault}
        />
      </main>
    </>
  );
}

export default function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <Dashboard />
        </ToastProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
