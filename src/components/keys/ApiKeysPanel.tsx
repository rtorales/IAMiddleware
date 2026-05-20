import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Provider {
  id: string;
  label: string;
  icon: string;
  docsUrl: string;
  placeholder: string;
}

const PROVIDERS: Provider[] = [
  {
    id: "openai",
    label: "OpenAI",
    icon: "⚡",
    docsUrl: "platform.openai.com/api-keys",
    placeholder: "sk-proj-...",
  },
  {
    id: "anthropic",
    label: "Anthropic",
    icon: "🔷",
    docsUrl: "console.anthropic.com/settings/keys",
    placeholder: "sk-ant-...",
  },
  {
    id: "deepseek",
    label: "DeepSeek",
    icon: "🔵",
    docsUrl: "platform.deepseek.com/api_keys",
    placeholder: "sk-...",
  },
  {
    id: "gemini",
    label: "Google Gemini",
    icon: "🟣",
    docsUrl: "aistudio.google.com/app/apikey",
    placeholder: "AIza...",
  },
];

interface ProviderState {
  configured: boolean;
  loading: boolean;
  saving: boolean;
  value: string;
  feedback: { ok: boolean; msg: string } | null;
}

function makeInitialState(): Record<string, ProviderState> {
  return Object.fromEntries(
    PROVIDERS.map((p) => [
      p.id,
      { configured: false, loading: true, saving: false, value: "", feedback: null },
    ])
  );
}

export function ApiKeysPanel() {
  const [states, setStates] = useState<Record<string, ProviderState>>(makeInitialState);

  useEffect(() => {
    PROVIDERS.forEach(({ id }) => {
      invoke<boolean>("check_api_key", { provider: id })
        .then((configured) =>
          setStates((prev) => ({
            ...prev,
            [id]: { ...prev[id], configured, loading: false },
          }))
        )
        .catch(() =>
          setStates((prev) => ({
            ...prev,
            [id]: { ...prev[id], loading: false },
          }))
        );
    });
  }, []);

  const handleSave = async (providerId: string) => {
    const key = states[providerId].value.trim();
    if (!key) return;

    setStates((prev) => ({
      ...prev,
      [providerId]: { ...prev[providerId], saving: true, feedback: null },
    }));

    try {
      await invoke("store_api_key", { provider: providerId, apiKey: key });
      setStates((prev) => ({
        ...prev,
        [providerId]: {
          ...prev[providerId],
          saving: false,
          configured: true,
          value: "",
          feedback: { ok: true, msg: "Key guardada correctamente" },
        },
      }));
    } catch (err) {
      setStates((prev) => ({
        ...prev,
        [providerId]: {
          ...prev[providerId],
          saving: false,
          feedback: {
            ok: false,
            msg: err instanceof Error ? err.message : "Error al guardar",
          },
        },
      }));
    }
  };

  return (
    <div className="max-w-2xl space-y-4">
      <div>
        <h2 className="text-lg font-semibold text-white">API Keys de Proveedores</h2>
        <p className="text-sm text-slate-400 mt-0.5">
          Las claves se cifran con AES-256-GCM en la bóveda local. Nunca salen del dispositivo.
        </p>
      </div>

      {PROVIDERS.map((provider) => {
        const s = states[provider.id];
        return (
          <div
            key={provider.id}
            className="bg-surface-800 border border-slate-800 rounded-xl p-5"
          >
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                <span className="text-xl">{provider.icon}</span>
                <span className="font-medium text-white">{provider.label}</span>
              </div>
              {s.loading ? (
                <span className="text-xs text-slate-500 animate-pulse">Verificando...</span>
              ) : s.configured ? (
                <span className="text-xs bg-emerald-900/50 text-emerald-400 border border-emerald-800 px-2 py-0.5 rounded-full">
                  Configurada
                </span>
              ) : (
                <span className="text-xs bg-slate-800 text-slate-500 border border-slate-700 px-2 py-0.5 rounded-full">
                  Sin configurar
                </span>
              )}
            </div>

            <div className="flex gap-2">
              <input
                type="password"
                value={s.value}
                onChange={(e) =>
                  setStates((prev) => ({
                    ...prev,
                    [provider.id]: { ...prev[provider.id], value: e.target.value, feedback: null },
                  }))
                }
                onKeyDown={(e) => e.key === "Enter" && handleSave(provider.id)}
                placeholder={s.configured ? "••••••••  (reemplazar)" : provider.placeholder}
                disabled={s.saving}
                className="flex-1 px-3 py-2 bg-surface-900 border border-slate-700 rounded-lg
                           text-sm text-white placeholder-slate-600 focus:outline-none
                           focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500
                           disabled:opacity-50 transition-colors font-mono"
              />
              <button
                onClick={() => handleSave(provider.id)}
                disabled={s.saving || !s.value.trim()}
                className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-700
                           disabled:text-slate-500 text-white text-sm font-medium rounded-lg
                           transition-colors focus:outline-none focus:ring-2 focus:ring-emerald-500"
              >
                {s.saving ? "Guardando..." : "Guardar"}
              </button>
            </div>

            {s.feedback && (
              <p
                className={`text-xs mt-2 ${
                  s.feedback.ok ? "text-emerald-400" : "text-red-400"
                }`}
              >
                {s.feedback.ok ? "✓" : "✗"} {s.feedback.msg}
              </p>
            )}

            <p className="text-xs text-slate-600 mt-2">{provider.docsUrl}</p>
          </div>
        );
      })}

      <div className="bg-surface-800 border border-slate-800 rounded-xl p-5 opacity-60">
        <div className="flex items-center gap-2">
          <span className="text-xl">🦙</span>
          <span className="font-medium text-white">Ollama</span>
          <span className="text-xs bg-sky-900/50 text-sky-400 border border-sky-800 px-2 py-0.5 rounded-full ml-auto">
            Local — sin key
          </span>
        </div>
        <p className="text-xs text-slate-500 mt-3">
          Ollama corre localmente en localhost:11434. No requiere API key.
        </p>
      </div>
    </div>
  );
}
