import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import type { RequestEntry } from "../../types";

interface LatencyChartProps {
  requests: RequestEntry[];
}

export function LatencyChart({ requests }: LatencyChartProps) {
  const data = requests
    .slice(-50)
    .map((r, i) => ({
      index: i + 1,
      latency: r.latencyMs,
      provider: r.provider,
    }));

  return (
    <div className="bg-surface-800 border border-slate-800 rounded-xl p-4">
      <h3 className="text-sm font-medium text-slate-300 mb-4">Latencia (ms) — Últimas peticiones</h3>
      {data.length === 0 ? (
        <div className="flex items-center justify-center h-40 text-slate-600 text-sm">
          Sin datos aún
        </div>
      ) : (
        <ResponsiveContainer width="100%" height={160}>
          <LineChart data={data} margin={{ top: 4, right: 4, bottom: 4, left: 0 }}>
            <XAxis dataKey="index" tick={false} axisLine={false} tickLine={false} />
            <YAxis
              tick={{ fill: "#64748b", fontSize: 10 }}
              axisLine={false}
              tickLine={false}
              tickFormatter={(v) => `${v}ms`}
              width={52}
            />
            <Tooltip
              contentStyle={{
                background: "#1e293b",
                border: "1px solid #334155",
                borderRadius: "8px",
                fontSize: "12px",
              }}
              formatter={(v: number) => [`${v}ms`, "Latencia"]}
            />
            <Line
              type="monotone"
              dataKey="latency"
              stroke="#3b82f6"
              strokeWidth={1.5}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      )}
    </div>
  );
}
