import type { MetricsSummary } from "../../types";

interface TokenCounterProps {
  summary: MetricsSummary | null;
}

export function TokenCounter({ summary }: TokenCounterProps) {
  return (
    <div className="bg-surface-800 border border-slate-800 rounded-xl p-4">
      <h3 className="text-sm font-medium text-slate-300 mb-4">Tokens Consumidos</h3>
      <div className="space-y-3">
        {summary?.byProvider.map((p) => {
          const totalAllProviders = summary.byProvider.reduce((a, b) => a + b.totalTokens, 0);
          const pct = totalAllProviders > 0 ? (p.totalTokens / totalAllProviders) * 100 : 0;
          return (
            <div key={p.provider}>
              <div className="flex justify-between items-center mb-1">
                <span className="text-xs text-slate-400 capitalize">{p.provider}</span>
                <span className="text-xs text-slate-300 font-mono">
                  {p.totalTokens.toLocaleString()}
                </span>
              </div>
              <div className="w-full bg-slate-700 rounded-full h-1.5">
                <div
                  className="bg-emerald-500 h-1.5 rounded-full transition-all duration-500"
                  style={{ width: `${pct}%` }}
                />
              </div>
            </div>
          );
        }) ?? (
          <div className="text-slate-600 text-xs text-center py-8">Sin datos aún</div>
        )}
      </div>
    </div>
  );
}
