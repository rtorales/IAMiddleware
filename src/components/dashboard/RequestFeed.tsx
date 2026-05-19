import { StatusBadge } from "../common/StatusBadge";
import type { RequestEntry } from "../../types";

interface RequestFeedProps {
  requests: RequestEntry[];
}

export function RequestFeed({ requests }: RequestFeedProps) {
  return (
    <div className="bg-surface-800 border border-slate-800 rounded-xl p-4">
      <h3 className="text-sm font-medium text-slate-300 mb-3">Feed de Peticiones</h3>
      <div className="space-y-1.5 max-h-64 overflow-y-auto pr-1">
        {requests.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-slate-600 text-sm">
            Esperando peticiones al proxy :12434
          </div>
        ) : (
          requests.slice().reverse().map((req) => (
            <div
              key={req.id}
              className="flex items-center gap-3 px-3 py-2 rounded-lg bg-surface-700 hover:bg-slate-700/50 transition-colors"
            >
              <StatusBadge status={req.status} />
              <span className="text-xs text-slate-400 capitalize flex-shrink-0">{req.provider}</span>
              <span className="text-xs text-slate-300 font-mono truncate flex-1">{req.model}</span>
              <span className="text-xs text-slate-500 flex-shrink-0">{req.latencyMs}ms</span>
              <span className="text-xs text-emerald-400 font-mono flex-shrink-0">
                ${req.costUsd.toFixed(5)}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
