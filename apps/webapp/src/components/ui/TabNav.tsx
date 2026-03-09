export const TAB_KEYS = {
  DEPOSIT: "deposit",
  BORROW: "borrow",
  REPAY: "repay",
} as const;

export type TabKey = (typeof TAB_KEYS)[keyof typeof TAB_KEYS];

const TAB_LABELS: Record<TabKey, string> = {
  [TAB_KEYS.DEPOSIT]: "Deposit",
  [TAB_KEYS.BORROW]: "Borrow",
  [TAB_KEYS.REPAY]: "Repay & Withdraw",
};

interface TabNavProps {
  activeTab: TabKey;
  onTabChange: (tab: TabKey) => void;
}

export function TabNav({ activeTab, onTabChange }: TabNavProps) {
  const tabs = [TAB_KEYS.DEPOSIT, TAB_KEYS.BORROW, TAB_KEYS.REPAY] as const;

  return (
    <nav className="flex items-center gap-1">
      {tabs.map((tab) => (
        <button
          key={tab}
          onClick={() => onTabChange(tab)}
          className={`rounded-lg px-3 py-1.5 text-sm font-medium transition-colors duration-200 cursor-pointer ${
            activeTab === tab
              ? "bg-surface-800 text-white"
              : "text-surface-400 hover:text-surface-200 hover:bg-surface-800/50"
          }`}
        >
          {TAB_LABELS[tab]}
        </button>
      ))}
    </nav>
  );
}
