import { useState } from "react";
import { useMetrics } from "../../hooks/useMetrics";
import type { RequestEntry } from "../../types";

const PROVIDERS = ["Todos", "openai", "anthropic", "deepseek", "gemini", "ollama"];

const statusStyle: Record<string, string> = {
  success:  "bg-emerald-900/50 text-emerald-400 border-emerald-800",
  error:    "bg-red-900/50 text-red-400 border-red-800",
  fallback: "bg-yellow-900/50 text-yellow-400 border-yellow-800",
  timeout:  "bg-slate-800 text-slate-400 border-slate-700",
};

const providerIcon: Record<string, string> = {
  openai:    "⚡",
  anthropic: "🔷",
  deepseek:  "🔵",
  gemini:    "🟣",
  ollama:    "🦙",
};

function formatTime(ms: number) {
  return new Date(ms).toLocaleTimeString("es", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

function Row({ req }: { req: RequestEntry }) {
  return (
    <tr className="border-b border-slate-800 hover:bg-slate-800/30 transition-colors">
      <td className="px-4 py-2.5 text-xs text-slate-400 whitespace-nowrap">
        {formatTime(req.createdAt)}
      </td>
      <td className="px-4 py-2.5">
        <span className="flex items-center gap-1.5 text-sm text-slate-300">
          <span>{providerIcon[req.provider] ?? "🤖"}</span>
          {req.provider}
        </span>
      </td>
      <td className="px-4 py-2.5 text-xs text-slate-400 font-mono max-w-[180px] truncate">
        {req.model}
      </td>
      <td className="px-4 py-2.5 text-xs text-slate-300 text-right">
        {(req.promptTokens + req.completionTokens).toLocaleString()}
      </td>
      <td className="px-4 py-2.5 text-xs text-right">
        <span className={req.costUsd > 0 ? "text-amber-400" : "text-slate-500"}>
          ${req.costUsd.toFixed(5)}
        </span>
      </td>
      <td className="px-4 py-2.5 text-xs text-slate-400 text-right">
        {req.latencyMs > 0 ? `${req.latencyMs} ms` : "—"}
      </td>
      <td className="px-4 py-2.5">
        <span className={`text-xs border px-2 py-0.5 rounded-full ${statusStyle[req.status] ?? statusStyle.timeout}`}>
          {req.status}
        </span>
      </td>
    </tr>
  );
}

export function RequestsPanel() {
  const { recentRequests, isLoading } = useMetrics("day");
  const [filter, setFilter] = useState("Todos");

  const filtered = filter === "Todos"
    ? recentRequests
    : recentRequests.filter((r) => r.provider === filter);

  const totalCost = filtered.reduce((acc, r) => acc + r.costUsd, 0);
  const totalTokens = filtered.reduce((acc, r) => acc + r.promptTokens + r.completionTokens, 0);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Peticiones recientes</h2>
          <p className="text-sm text-slate-400 mt-0.5">Últimas 50 peticiones procesadas por el proxy.</p>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-slate-500">Filtrar:</span>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            className="px-3 py-1.5 bg-surface-800 border border-slate-700 rounded-lg text-sm
                       text-white focus:outline-none focus:border-emerald-500"
          >
            {PROVIDERS.map((p) => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Summary strip */}
      <div className="flex gap-4 text-xs text-slate-400">
        <span><span className="text-white font-medium">{filtered.length}</span> peticiones</span>
        <span><span className="text-amber-400 font-medium">${totalCost.toFixed(5)}</span> costo total</span>
        <span><span className="text-sky-400 font-medium">{totalTokens.toLocaleString()}</span> tokens</span>
      </div>

      <div className="bg-surface-800 border border-slate-800 rounded-xl overflow-hidden">
        {isLoading ? (
          <div className="flex items-center justify-center h-40 text-slate-500 text-sm animate-pulse">
            Cargando...
          </div>
        ) : filtered.length === 0 ? (
          <div className="flex items-center justify-center h-40 text-slate-500 text-sm">
            Sin peticiones registradas. Envía una llamada al proxy en :12434.
          </div>
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b border-slate-800 bg-slate-900/50">
                <th className="px-4 py-2.5 text-xs text-slate-500 text-left font-medium">Hora</th>
                <th className="px-4 py-2.5 text-xs text-slate-500 text-left font-medium">Proveedor</th>
                <th className="px-4 py-2.5 text-xs text-slate-500 text-left font-medium">Modelo</th>
                <th className="px-4 py-2.5 text-xs text-slate-500 text-right font-medium">Tokens</th>
                <th className="px-4 py-2.5 text-xs text-slate-500 text-right font-medium">Costo</th>
                <th className="px-4 py-2.5 text-xs text-slate-500 text-right font-medium">Latencia</th>
                <th className="px-4 py-2.5 text-xs text-slate-500 text-left font-medium">Estado</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((req) => (
                <Row key={req.id} req={req} />
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
