import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import type { ProviderMetrics } from "../../types";

const PROVIDER_COLORS: Record<string, string> = {
  openai:    "#10b981",
  anthropic: "#8b5cf6",
  gemini:    "#3b82f6",
  deepseek:  "#f59e0b",
  ollama:    "#06b6d4",
};

interface CostTrackerProps {
  data: ProviderMetrics[];
}

export function CostTracker({ data }: CostTrackerProps) {
  const chartData = data.map((d) => ({
    name: d.provider.charAt(0).toUpperCase() + d.provider.slice(1),
    cost: parseFloat(d.totalCostUsd.toFixed(6)),
    provider: d.provider,
  }));

  return (
    <div className="bg-surface-800 border border-slate-800 rounded-xl p-4">
      <h3 className="text-sm font-medium text-slate-300 mb-4">Costo por Proveedor (Hoy)</h3>
      {chartData.length === 0 ? (
        <div className="flex items-center justify-center h-40 text-slate-600 text-sm">
          Sin datos aún
        </div>
      ) : (
        <ResponsiveContainer width="100%" height={160}>
          <BarChart data={chartData} margin={{ top: 4, right: 4, bottom: 4, left: 0 }}>
            <XAxis
              dataKey="name"
              tick={{ fill: "#94a3b8", fontSize: 11 }}
              axisLine={false}
              tickLine={false}
            />
            <YAxis
              tick={{ fill: "#64748b", fontSize: 10 }}
              axisLine={false}
              tickLine={false}
              tickFormatter={(v) => `$${v}`}
              width={48}
            />
            <Tooltip
              contentStyle={{
                background: "#1e293b",
                border: "1px solid #334155",
                borderRadius: "8px",
                fontSize: "12px",
              }}
              formatter={(v: number) => [`$${v.toFixed(6)}`, "Costo"]}
            />
            <Bar dataKey="cost" radius={[4, 4, 0, 0]}>
              {chartData.map((entry) => (
                <Cell
                  key={entry.provider}
                  fill={PROVIDER_COLORS[entry.provider] ?? "#64748b"}
                />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      )}
    </div>
  );
}
