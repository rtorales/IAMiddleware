import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface VaultState {
  isUnlocked: boolean;
  isLoading: boolean;
  error: string | null;
}

export function useVault() {
  const [state, setState] = useState<VaultState>({
    isUnlocked: false,
    isLoading: true,
    error: null,
  });

  useEffect(() => {
    // Check initial vault status on mount
    invoke<{ unlocked: boolean }>("get_vault_status")
      .then((status) => {
        setState({ isUnlocked: status.unlocked, isLoading: false, error: null });
      })
      .catch(() => {
        // Vault not initialized or OS keychain unavailable — needs unlock
        setState({ isUnlocked: false, isLoading: false, error: null });
      });

    // Listen for vault lock events (session timeout or OS lock)
    const unlisten = listen("vault_locked", () => {
      setState((prev) => ({ ...prev, isUnlocked: false }));
    });

    return () => {
      unlisten.then((unlistenFn: () => void) => unlistenFn());
    };
  }, []);

  const unlock = async (masterPassword: string): Promise<boolean> => {
    try {
      await invoke("unlock_vault", { masterPassword });
      setState((prev) => ({ ...prev, isUnlocked: true, error: null }));
      return true;
    } catch (err) {
      setState((prev) => ({
        ...prev,
        error: err instanceof Error ? err.message : "Error al desbloquear bóveda",
      }));
      return false;
    }
  };

  const lock = async () => {
    await invoke("lock_vault").catch(() => {});
    setState((prev) => ({ ...prev, isUnlocked: false }));
  };

  return { ...state, unlock, lock };
}
