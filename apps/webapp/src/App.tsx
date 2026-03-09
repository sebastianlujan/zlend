import { useState } from "react";
import { WagmiProvider, useAccount } from "wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { config } from "./config/wagmi.ts";
import { Header } from "./components/layout/Header.tsx";
import { SummaryStrip } from "./components/dashboard/SummaryStrip.tsx";
import { DepositView } from "./components/views/DepositView.tsx";
import { BorrowView } from "./components/views/BorrowView.tsx";
import { RepayWithdrawView } from "./components/views/RepayWithdrawView.tsx";
import { TAB_KEYS, type TabKey } from "./components/ui/TabNav.tsx";
import { useVaults } from "./hooks/useVaults.ts";
import { Button } from "./components/ui/Button.tsx";

const queryClient = new QueryClient();

function Dashboard() {
  const { isConnected } = useAccount();
  const { vaults, addVault, updateVault } = useVaults();
  const [activeTab, setActiveTab] = useState<TabKey>(TAB_KEYS.DEPOSIT);

  if (!isConnected) {
    return (
      <>
        <Header activeTab={activeTab} onTabChange={setActiveTab} />
        <main className="mx-auto max-w-5xl px-6 py-20 text-center">
          <h2 className="text-2xl font-bold text-white">
            Private lending, powered by ZK proofs
          </h2>
          <p className="mt-3 text-surface-400">
            Borrow USDT against your ZEC holdings — without revealing your
            identity.
          </p>
          <div className="mt-8">
            <Button size="lg" onClick={() => {}}>
              Connect Wallet
            </Button>
          </div>
        </main>
      </>
    );
  }

  return (
    <>
      <Header activeTab={activeTab} onTabChange={setActiveTab} />
      <main className="mx-auto max-w-5xl px-6 py-8">
        <div className="mb-8">
          <SummaryStrip vaults={vaults} />
        </div>

        {activeTab === TAB_KEYS.DEPOSIT && (
          <DepositView vaults={vaults} onAddVault={addVault} />
        )}

        {activeTab === TAB_KEYS.BORROW && (
          <BorrowView
            vaults={vaults}
            onUpdateVault={updateVault}
            onTabChange={setActiveTab}
          />
        )}

        {activeTab === TAB_KEYS.REPAY && (
          <RepayWithdrawView
            vaults={vaults}
            onUpdateVault={updateVault}
            onTabChange={setActiveTab}
          />
        )}
      </main>
    </>
  );
}

export default function App() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <Dashboard />
      </QueryClientProvider>
    </WagmiProvider>
  );
}
