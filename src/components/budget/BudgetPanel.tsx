import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBudget } from "../../hooks/useBudget";
import type { BudgetStatus } from "../../types";

const PERIODS = ["daily", "weekly", "monthly"] as const;
type Period = typeof PERIODS[number];

const periodLabel: Record<Period, string> = {
  daily: "Diario",
  weekly: "Semanal",
  monthly: "Mensual",
};

function statusColor(b: BudgetStatus) {
  if (b.locked)            return { bar: "bg-red-500",    badge: "bg-red-900/50 text-red-400 border-red-800",    label: "Bloqueado" };
  if (b.percentUsed >= 0.85) return { bar: "bg-orange-500", badge: "bg-orange-900/50 text-orange-400 border-orange-800", label: "Alerta 85%" };
  if (b.percentUsed >= 0.60) return { bar: "bg-yellow-500", badge: "bg-yellow-900/50 text-yellow-400 border-yellow-800", label: "Alerta 60%" };
  return { bar: "bg-emerald-500", badge: "bg-emerald-900/50 text-emerald-400 border-emerald-800", label: "Normal" };
}

function BudgetCard({ budget, onRefresh }: { budget: BudgetStatus; onRefresh: () => void }) {
  const pct = Math.min(budget.percentUsed * 100, 100);
  const colors = statusColor(budget);

  return (
    <div className="bg-surface-800 border border-slate-800 rounded-xl p-5">
      <div className="flex items-center justify-between mb-3">
        <div>
          <span className="font-medium text-white">{budget.projectTag}</span>
          <span className="ml-2 text-xs text-slate-500">{periodLabel[budget.period as Period] ?? budget.period}</span>
        </div>
        <span className={`text-xs border px-2 py-0.5 rounded-full ${colors.badge}`}>
          {colors.label}
        </span>
      </div>

      <div className="mb-2">
        <div className="flex justify-between text-xs text-slate-400 mb-1">
          <span>${budget.spentUsd.toFixed(4)} gastado</span>
          <span>${budget.limitUsd.toFixed(2)} límite</span>
        </div>
        <div className="h-2 bg-slate-700 rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all ${colors.bar}`}
            style={{ width: `${pct}%` }}
          />
        </div>
        <p className="text-right text-xs text-slate-500 mt-1">{pct.toFixed(1)}%</p>
      </div>

      {budget.locked && (
        <p className="text-xs text-red-400 mt-2">
          Presupuesto agotado — las peticiones de este proyecto están bloqueadas.
        </p>
      )}
    </div>
  );
}

interface NewBudgetForm {
  projectTag: string;
  limitUsd: string;
  period: Period;
}

export function BudgetPanel() {
  const { budgets } = useBudget();
  const [showForm, setShowForm] = useState(false);
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [form, setForm] = useState<NewBudgetForm>({
    projectTag: "",
    limitUsd: "",
    period: "monthly",
  });

  const handleCreate = async () => {
    const limit = parseFloat(form.limitUsd);
    if (!form.projectTag.trim() || isNaN(limit) || limit <= 0) return;

    setSaving(true);
    setFeedback(null);
    try {
      await invoke("create_budget", {
        projectTag: form.projectTag.trim(),
        limitUsd: limit,
        period: form.period,
      });
      setFeedback("Presupuesto guardado");
      setForm({ projectTag: "", limitUsd: "", period: "monthly" });
      setShowForm(false);
    } catch (err) {
      setFeedback(err instanceof Error ? err.message : "Error al guardar");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="max-w-2xl space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Presupuestos</h2>
          <p className="text-sm text-slate-400 mt-0.5">
            Límites de gasto por proyecto. Alertas automáticas al 60%, 85% y 100%.
          </p>
        </div>
        <button
          onClick={() => { setShowForm((v) => !v); setFeedback(null); }}
          className="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white text-sm
                     font-medium rounded-lg transition-colors"
        >
          {showForm ? "Cancelar" : "+ Nuevo"}
        </button>
      </div>

      {showForm && (
        <div className="bg-surface-800 border border-slate-700 rounded-xl p-5 space-y-3">
          <h3 className="text-sm font-medium text-white">Nuevo presupuesto</h3>
          <div className="grid grid-cols-3 gap-3">
            <input
              type="text"
              placeholder="proyecto (ej: default)"
              value={form.projectTag}
              onChange={(e) => setForm((f) => ({ ...f, projectTag: e.target.value }))}
              className="col-span-1 px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                         text-sm text-white placeholder-slate-600 focus:outline-none
                         focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500"
            />
            <input
              type="number"
              placeholder="límite USD"
              min="0.01"
              step="0.01"
              value={form.limitUsd}
              onChange={(e) => setForm((f) => ({ ...f, limitUsd: e.target.value }))}
              className="col-span-1 px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                         text-sm text-white placeholder-slate-600 focus:outline-none
                         focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500"
            />
            <select
              value={form.period}
              onChange={(e) => setForm((f) => ({ ...f, period: e.target.value as Period }))}
              className="col-span-1 px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                         text-sm text-white focus:outline-none focus:border-emerald-500
                         focus:ring-1 focus:ring-emerald-500"
            >
              {PERIODS.map((p) => (
                <option key={p} value={p}>{periodLabel[p]}</option>
              ))}
            </select>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={handleCreate}
              disabled={saving || !form.projectTag.trim() || !form.limitUsd}
              className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-700
                         disabled:text-slate-500 text-white text-sm font-medium rounded-lg transition-colors"
            >
              {saving ? "Guardando..." : "Guardar"}
            </button>
            {feedback && (
              <p className="text-xs text-emerald-400">{feedback}</p>
            )}
          </div>
        </div>
      )}

      {budgets.length === 0 ? (
        <div className="flex items-center justify-center h-40 text-slate-500 text-sm">
          No hay presupuestos configurados. Crea uno con "+ Nuevo".
        </div>
      ) : (
        <div className="space-y-3">
          {budgets.map((b) => (
            <BudgetCard key={b.projectTag} budget={b} onRefresh={() => {}} />
          ))}
        </div>
      )}
    </div>
  );
}
