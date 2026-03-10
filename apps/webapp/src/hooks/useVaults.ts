import { useState, useEffect } from "react";
import { useAccount } from "wagmi";
import { VAULT_STATUS, type Vault } from "../types/vault.ts";

const STORAGE_KEY = "ogbank-vaults";

function loadVaults(address: string): Vault[] {
  try {
    const raw = localStorage.getItem(`${STORAGE_KEY}-${address}`);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveVaults(address: string, vaults: Vault[]) {
  localStorage.setItem(`${STORAGE_KEY}-${address}`, JSON.stringify(vaults));
}

function generateId(): string {
  return crypto.randomUUID().replace(/-/g, "").slice(0, 12);
}

function generateFieldElement(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  bytes[0] = 0;
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function generateShieldedAddress(): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  let addr = "zs1";
  for (let i = 0; i < 60; i++) {
    addr += chars[Math.floor(Math.random() * chars.length)];
  }
  return addr;
}

export function useVaults() {
  const { address } = useAccount();
  const [vaults, setVaults] = useState<Vault[]>([]);

  useEffect(() => {
    if (address) {
      setVaults(loadVaults(address));
    } else {
      setVaults([]);
    }
  }, [address]);

  const persist = (updated: Vault[]) => {
    setVaults(updated);
    if (address) saveVaults(address, updated);
  };

  const addVault = (zecAmount: string, shieldedAddress: string): string => {
    const vault: Vault = {
      id: generateId(),
      zecAmount,
      shieldedAddress: shieldedAddress || generateShieldedAddress(),
      userSecret: generateFieldElement(),
      nonce: generateFieldElement(),
      status: VAULT_STATUS.DEPOSITED,
      createdAt: Date.now(),
    };
    persist([vault, ...vaults]);
    return vault.id;
  };

  const updateVault = (id: string, updates: Partial<Vault>) => {
    persist(
      vaults.map((v) => (v.id === id ? { ...v, ...updates } : v)),
    );
  };

  const getDepositAddress = (): string => {
    return generateShieldedAddress();
  };

  return { vaults, addVault, updateVault, getDepositAddress };
}
