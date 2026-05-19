import type { BudgetAlert } from "../../types";

interface AlertBannerProps {
  alerts: BudgetAlert[];
  onDismiss: (index: number) => void;
}

const levelConfig = {
  warning_60: { bg: "bg-yellow-950/80", border: "border-yellow-700", text: "text-yellow-300", icon: "⚠️" },
  warning_85: { bg: "bg-orange-950/80", border: "border-orange-700", text: "text-orange-300", icon: "🔶" },
  hard_lock:  { bg: "bg-red-950/80",    border: "border-red-700",    text: "text-red-300",    icon: "🔒" },
};

export function AlertBanner({ alerts, onDismiss }: AlertBannerProps) {
  if (alerts.length === 0) return null;

  return (
    <div className="space-y-2 px-4 pt-3">
      {alerts.map((alert, i) => {
        const cfg = levelConfig[alert.level];
        return (
          <div key={i} className={`flex items-start gap-3 px-4 py-3 rounded-lg border ${cfg.bg} ${cfg.border}`}>
            <span className="text-lg leading-none mt-0.5">{cfg.icon}</span>
            <div className="flex-1 min-w-0">
              <p className={`text-sm font-medium ${cfg.text}`}>
                Presupuesto <strong>{alert.projectTag}</strong> al {alert.percentUsed.toFixed(0)}%
              </p>
              <p className="text-xs text-slate-400 mt-0.5">
                ${alert.spentUsd.toFixed(4)} / ${alert.limitUsd.toFixed(2)} USD
              </p>
            </div>
            <button
              onClick={() => onDismiss(i)}
              className="text-slate-500 hover:text-slate-300 text-lg leading-none flex-shrink-0"
            >
              ×
            </button>
          </div>
        );
      })}
    </div>
  );
}
