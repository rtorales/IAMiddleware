import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { RoutingConfig } from "../../types";

const DEFAULT: RoutingConfig = {
  tau: 0.5,
  ollamaRouterUrl: "http://localhost:11434",
  routerModel: "qwen2.5:3b",
  classifierModel: "qwen2.5:1.5b",
  complexityThresholdLow: 0.3,
  complexityThresholdHigh: 0.7,
  fallbackToLocal: true,
};

function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="space-y-1.5">
      <label className="block text-sm font-medium text-slate-300">{label}</label>
      {children}
      {hint && <p className="text-xs text-slate-500">{hint}</p>}
    </div>
  );
}

export function SettingsPanel() {
  const [config, setConfig] = useState<RoutingConfig>(DEFAULT);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<{ ok: boolean; msg: string } | null>(null);

  useEffect(() => {
    invoke<RoutingConfig>("get_routing_config")
      .then(setConfig)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const set = <K extends keyof RoutingConfig>(key: K, value: RoutingConfig[K]) =>
    setConfig((c) => ({ ...c, [key]: value }));

  const handleSave = async () => {
    setSaving(true);
    setFeedback(null);
    try {
      await invoke("save_routing_config", {
        tau: config.tau,
        ollamaRouterUrl: config.ollamaRouterUrl,
        routerModel: config.routerModel,
        classifierModel: config.classifierModel,
        complexityThresholdLow: config.complexityThresholdLow,
        complexityThresholdHigh: config.complexityThresholdHigh,
        fallbackToLocal: config.fallbackToLocal,
      });
      setFeedback({ ok: true, msg: "Configuración guardada" });
    } catch (err) {
      setFeedback({ ok: false, msg: err instanceof Error ? err.message : "Error al guardar" });
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-40 text-slate-500 text-sm animate-pulse">
        Cargando configuración...
      </div>
    );
  }

  return (
    <div className="max-w-xl space-y-6">
      <div>
        <h2 className="text-lg font-semibold text-white">Configuración del Router</h2>
        <p className="text-sm text-slate-400 mt-0.5">
          Parámetros del router semántico local (Ollama). Controlan cómo se decide entre modelos locales y cloud.
        </p>
      </div>

      <div className="bg-surface-800 border border-slate-800 rounded-xl p-5 space-y-5">
        <h3 className="text-sm font-semibold text-slate-300 uppercase tracking-wider">Ollama</h3>

        <Field label="URL del servidor Ollama" hint="Por defecto: http://localhost:11434">
          <input
            type="text"
            value={config.ollamaRouterUrl}
            onChange={(e) => set("ollamaRouterUrl", e.target.value)}
            className="w-full px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                       text-sm text-white font-mono focus:outline-none focus:border-emerald-500
                       focus:ring-1 focus:ring-emerald-500"
          />
        </Field>

        <div className="grid grid-cols-2 gap-4">
          <Field label="Modelo router" hint="Clasifica complejidad de la petición">
            <input
              type="text"
              value={config.routerModel}
              onChange={(e) => set("routerModel", e.target.value)}
              className="w-full px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                         text-sm text-white font-mono focus:outline-none focus:border-emerald-500
                         focus:ring-1 focus:ring-emerald-500"
            />
          </Field>
          <Field label="Modelo clasificador" hint="Identifica tipo de tarea">
            <input
              type="text"
              value={config.classifierModel}
              onChange={(e) => set("classifierModel", e.target.value)}
              className="w-full px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                         text-sm text-white font-mono focus:outline-none focus:border-emerald-500
                         focus:ring-1 focus:ring-emerald-500"
            />
          </Field>
        </div>
      </div>

      <div className="bg-surface-800 border border-slate-800 rounded-xl p-5 space-y-5">
        <h3 className="text-sm font-semibold text-slate-300 uppercase tracking-wider">Umbral τ (Tau)</h3>

        <Field
          label={`τ global = ${config.tau.toFixed(2)}`}
          hint="Controla el balance local↔cloud. 0 = todo local, 1 = todo cloud."
        >
          <input
            type="range"
            min={0} max={1} step={0.01}
            value={config.tau}
            onChange={(e) => set("tau", parseFloat(e.target.value))}
            className="w-full accent-emerald-500"
          />
          <div className="flex justify-between text-xs text-slate-600 mt-0.5">
            <span>0.0 — todo local</span>
            <span>1.0 — todo cloud</span>
          </div>
        </Field>

        <div className="grid grid-cols-2 gap-4">
          <Field label={`Umbral bajo = ${config.complexityThresholdLow.toFixed(2)}`} hint="Por debajo → Ollama local">
            <input
              type="range"
              min={0} max={1} step={0.01}
              value={config.complexityThresholdLow}
              onChange={(e) => set("complexityThresholdLow", parseFloat(e.target.value))}
              className="w-full accent-sky-500"
            />
          </Field>
          <Field label={`Umbral alto = ${config.complexityThresholdHigh.toFixed(2)}`} hint="Por encima → modelo cloud">
            <input
              type="range"
              min={0} max={1} step={0.01}
              value={config.complexityThresholdHigh}
              onChange={(e) => set("complexityThresholdHigh", parseFloat(e.target.value))}
              className="w-full accent-violet-500"
            />
          </Field>
        </div>
      </div>

      <div className="bg-surface-800 border border-slate-800 rounded-xl p-5">
        <label className="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            checked={config.fallbackToLocal}
            onChange={(e) => set("fallbackToLocal", e.target.checked)}
            className="w-4 h-4 accent-emerald-500"
          />
          <div>
            <span className="text-sm font-medium text-white">Fallback a local si falla el proveedor cloud</span>
            <p className="text-xs text-slate-500 mt-0.5">
              Si el proveedor cloud devuelve error, reintenta con Ollama automáticamente.
            </p>
          </div>
        </label>
      </div>

      <div className="flex items-center gap-4">
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-700
                     disabled:text-slate-500 text-white font-medium text-sm rounded-lg
                     transition-colors focus:outline-none focus:ring-2 focus:ring-emerald-500"
        >
          {saving ? "Guardando..." : "Guardar configuración"}
        </button>
        {feedback && (
          <p className={`text-sm ${feedback.ok ? "text-emerald-400" : "text-red-400"}`}>
            {feedback.ok ? "✓" : "✗"} {feedback.msg}
          </p>
        )}
      </div>
    </div>
  );
}
