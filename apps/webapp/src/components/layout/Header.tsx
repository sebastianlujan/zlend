import { ConnectButton } from "../wallet/ConnectButton.tsx";

export function Header() {
  return (
    <header className="sticky top-0 z-50 border-b border-surface-800 bg-surface-950/80 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-5xl items-center justify-between px-6">
        <span className="text-xl font-bold tracking-tight text-white">
          OG<span className="text-primary-500">Bank</span>
        </span>
        <ConnectButton />
      </div>
    </header>
  );
}
