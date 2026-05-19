import { create } from "zustand";
import type { ProviderInfo, RoutingConfig, ProxyStatus } from "../types";

// SECURITY: Este store solo contiene estado de UI — nunca secretos ni API keys.
interface AppState {
  providers: ProviderInfo[];
  proxyStatus: ProxyStatus | null;
  routingConfig: RoutingConfig | null;
  activeTab: "dashboard" | "requests" | "budgets" | "settings" | "keys";

  setProviders: (p: ProviderInfo[]) => void;
  setProxyStatus: (s: ProxyStatus) => void;
  setRoutingConfig: (c: RoutingConfig) => void;
  setActiveTab: (t: AppState["activeTab"]) => void;
}

export const useAppStore = create<AppState>((set) => ({
  providers: [],
  proxyStatus: null,
  routingConfig: null,
  activeTab: "dashboard",

  setProviders: (providers) => set({ providers }),
  setProxyStatus: (proxyStatus) => set({ proxyStatus }),
  setRoutingConfig: (routingConfig) => set({ routingConfig }),
  setActiveTab: (activeTab) => set({ activeTab }),
}));
