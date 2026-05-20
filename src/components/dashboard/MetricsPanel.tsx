import { useMetrics } from "../../hooks/useMetrics";
import { useBudget } from "../../hooks/useBudget";
import { AlertBanner } from "../common/AlertBanner";
import { CostTracker } from "./CostTracker";
import { LatencyChart } from "./LatencyChart";
import { TokenCounter } from "./TokenCounter";
import { RequestFeed } from "./RequestFeed";
import { ApiKeysPanel } from "../keys/ApiKeysPanel";
import { BudgetPanel } from "../budget/BudgetPanel";
import { RequestsPanel } from "../requests/RequestsPanel";
import { SettingsPanel } from "../settings/SettingsPanel";
import { useAppStore } from "../../stores/appStore";

export function Dashboard() {
  const { summary, recentRequests, isLoading } = useMetrics("day");
  const { alerts, dismissAlert } = useBudget();
  const { activeTab, setActiveTab } = useAppStore();

  const tabs = [
    { id: "dashboard" as const, label: "Dashboard" },
    { id: "requests" as const, label: "Peticiones" },
    { id: "budgets" as const, label: "Presupuestos" },
    { id: "settings" as const, label: "Configuración" },
    { id: "keys" as const, label: "Llaves" },
  ];

  return (
    <div className="flex flex-col min-h-screen bg-surface-900">
      {/* Header */}
      <header className="border-b border-slate-800 px-6 py-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-xl">🤖</span>
          <h1 className="text-lg font-bold text-white">IA Middleware</h1>
          <span className="text-xs bg-emerald-900/50 text-emerald-400 px-2 py-0.5 rounded-full border border-emerald-800">
            v0.1
          </span>
        </div>
        <div className="flex items-center gap-2 text-xs text-slate-500">
          <span className="w-2 h-2 bg-emerald-400 rounded-full animate-pulse" />
          Proxy activo :12434
        </div>
      </header>

      {/* Budget alerts */}
      <AlertBanner alerts={alerts} onDismiss={dismissAlert} />

      {/* Navigation tabs */}
      <nav className="flex gap-1 px-6 pt-3 border-b border-slate-800">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium rounded-t-lg transition-colors
              ${activeTab === tab.id
                ? "text-emerald-400 border-b-2 border-emerald-400 bg-emerald-950/20"
                : "text-slate-400 hover:text-slate-200"
              }`}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      {/* Main content */}
      <main className="flex-1 p-6 overflow-auto">
        {activeTab === "dashboard" && (
          <div className="space-y-6">
            {/* KPI row */}
            <div className="grid grid-cols-4 gap-4">
              <KpiCard
                label="Costo Hoy"
                value={summary ? `$${summary.totalCostUsd.toFixed(4)}` : "—"}
                sub="USD"
                loading={isLoading}
                color="text-emerald-400"
              />
              <KpiCard
                label="Tokens"
                value={summary ? summary.totalTokens.toLocaleString() : "—"}
                sub="Total"
                loading={isLoading}
                color="text-sky-400"
              />
              <KpiCard
                label="Peticiones"
                value={summary ? summary.requestCount.toString() : "—"}
                sub="Hoy"
                loading={isLoading}
                color="text-violet-400"
              />
              <KpiCard
                label="Tasa Éxito"
                value={summary ? `${(summary.successRate * 100).toFixed(1)}%` : "—"}
                sub="Últimas 24h"
                loading={isLoading}
                color="text-amber-400"
              />
            </div>

            {/* Charts row */}
            <div className="grid grid-cols-2 gap-4">
              <CostTracker data={summary?.byProvider ?? []} />
              <LatencyChart requests={recentRequests} />
            </div>

            {/* Token breakdown + feed */}
            <div className="grid grid-cols-3 gap-4">
              <TokenCounter summary={summary} />
              <div className="col-span-2">
                <RequestFeed requests={recentRequests} />
              </div>
            </div>
          </div>
        )}

        {activeTab === "keys"      && <ApiKeysPanel />}
        {activeTab === "budgets"   && <BudgetPanel />}
        {activeTab === "requests"  && <RequestsPanel />}
        {activeTab === "settings"  && <SettingsPanel />}
      </main>
    </div>
  );
}

function KpiCard({
  label,
  value,
  sub,
  loading,
  color,
}: {
  label: string;
  value: string;
  sub: string;
  loading: boolean;
  color: string;
}) {
  return (
    <div className="bg-surface-800 border border-slate-800 rounded-xl p-4">
      <p className="text-xs text-slate-500 uppercase tracking-wider">{label}</p>
      <p className={`text-2xl font-bold mt-1 ${loading ? "animate-pulse text-slate-600" : color}`}>
        {loading ? "···" : value}
      </p>
      <p className="text-xs text-slate-600 mt-0.5">{sub}</p>
    </div>
  );
}
