import { useState, useRef, useCallback, useEffect, useMemo } from "react";
import { animate } from "animejs";
import { ChevronDown, Vault as VaultIcon } from "lucide-react";
import { VaultCard } from "./VaultCard.tsx";
import { EmptyState } from "../ui/EmptyState.tsx";
import { usePagination } from "../../hooks/usePagination.ts";
import { VAULT_STATUS, STATUS_ORDER, type Vault } from "../../types/vault.ts";
import type { JSAnimation } from "animejs";

export function VaultList({ vaults, selectedVaultId, onSelectVault, autoOpen }: VaultListProps) {
  const [open, setOpen] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);
  const chevronRef = useRef<SVGSVGElement>(null);
  const animRef = useRef<JSAnimation | null>(null);
  const chevronAnimRef = useRef<JSAnimation | null>(null);

  const sortedVaults = useMemo(
    () => [...vaults].sort((a, b) => (STATUS_ORDER[a.status] ?? 9) - (STATUS_ORDER[b.status] ?? 9)),
    [vaults],
  );

  const { pageItems, page, totalPages, hasPrev, hasNext, prev, next } =
    usePagination(sortedVaults, 10);

  const selectedVault = selectedVaultId
    ? vaults.find((v) => v.id === selectedVaultId)
    : null;

  const toggle = useCallback(() => {
    const el = contentRef.current;
    if (!el) return;

    setOpen((prev) => {
      const willOpen = !prev;

      if (willOpen) {
        el.style.height = "auto";
        const targetHeight = el.scrollHeight;
        el.style.height = "0px";

        animRef.current?.pause();
        animRef.current = animate(el, {
          height: [0, targetHeight],
          opacity: [0, 1],
          duration: 300,
          ease: "outCubic",
          onComplete: () => {
            el.style.height = "auto";
          },
        });
      } else {
        animRef.current?.pause();
        animRef.current = animate(el, {
          height: [el.scrollHeight, 0],
          opacity: [1, 0],
          duration: 250,
          ease: "inCubic",
        });
      }

      chevronAnimRef.current?.pause();
      if (chevronRef.current) {
        chevronAnimRef.current = animate(chevronRef.current, {
          rotate: willOpen ? 180 : 0,
          duration: 300,
          ease: "outCubic",
        });
      }

      return willOpen;
    });
  }, []);

  useEffect(() => {
    if (autoOpen && !open) {
      toggle();
    }
  }, [autoOpen]);

  const isSelectable = (vault: Vault) =>
    vault.status !== VAULT_STATUS.WITHDRAWN;

  if (vaults.length === 0) {
    return (
      <EmptyState
        icon={VaultIcon}
        heading="No vaults yet"
        description="Deposit ZEC above to create your first vault and start borrowing."
      />
    );
  }

  return (
    <div>
      <button
        onClick={toggle}
        className="flex w-full cursor-pointer items-center justify-between rounded-lg border border-surface-700/50 bg-surface-900 px-5 py-3.5 transition-colors hover:bg-surface-800"
      >
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-white">My Vaults</span>
          {selectedVault && (
            <span className="font-mono text-xs text-primary-400">
              #{selectedVault.id.slice(0, 6)} selected
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <span className="rounded-full bg-surface-700 px-2 py-0.5 font-mono text-[10px] text-surface-300">
            {vaults.length}
          </span>
          <ChevronDown
            ref={chevronRef}
            className="h-4 w-4 text-surface-400"
          />
        </div>
      </button>

      <div
        ref={contentRef}
        style={{ height: 0, opacity: 0, overflow: "hidden" }}
      >
        <div className="space-y-2 pt-3">
          {pageItems.map((vault) => (
            <VaultCard
              key={vault.id}
              vault={vault}
              selected={vault.id === selectedVaultId}
              selectable={isSelectable(vault)}
              onSelect={onSelectVault}
            />
          ))}
        </div>

        {totalPages > 1 && (
          <div className="mt-3 flex items-center justify-center gap-3">
            <button
              onClick={prev}
              disabled={!hasPrev}
              className="rounded px-2 py-1 text-xs text-surface-500 hover:bg-surface-800 disabled:cursor-not-allowed disabled:opacity-30"
            >
              Prev
            </button>
            <span className="font-mono text-xs text-surface-500">
              {page + 1} / {totalPages}
            </span>
            <button
              onClick={next}
              disabled={!hasNext}
              className="rounded px-2 py-1 text-xs text-surface-500 hover:bg-surface-800 disabled:cursor-not-allowed disabled:opacity-30"
            >
              Next
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
