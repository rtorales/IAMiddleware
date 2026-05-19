import { useState, FormEvent } from "react";
import { useVault } from "../../hooks/useVault";

export function VaultUnlock() {
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { unlock, error } = useVault();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!password.trim()) return;
    setIsSubmitting(true);
    await unlock(password);
    setIsSubmitting(false);
    setPassword("");
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-surface-900">
      <div className="w-full max-w-sm mx-4">
        <div className="text-center mb-8">
          <div className="text-4xl mb-3">🔐</div>
          <h1 className="text-2xl font-bold text-white">IA Middleware</h1>
          <p className="text-slate-400 mt-1 text-sm">Desbloquea la bóveda para continuar</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="password" className="block text-sm font-medium text-slate-300 mb-1.5">
              Contraseña maestra
            </label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="••••••••••••"
              autoFocus
              disabled={isSubmitting}
              className="w-full px-4 py-2.5 bg-surface-700 border border-slate-600 rounded-lg
                         text-white placeholder-slate-500 focus:outline-none focus:border-emerald-500
                         focus:ring-1 focus:ring-emerald-500 disabled:opacity-50 transition-colors"
            />
          </div>

          {error && (
            <div className="text-red-400 text-sm bg-red-950/50 border border-red-800 rounded-lg px-3 py-2">
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={isSubmitting || !password.trim()}
            className="w-full py-2.5 px-4 bg-emerald-600 hover:bg-emerald-500 disabled:bg-slate-700
                       disabled:text-slate-500 text-white font-medium rounded-lg transition-colors
                       focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2
                       focus:ring-offset-surface-900"
          >
            {isSubmitting ? "Desbloqueando..." : "Desbloquear"}
          </button>
        </form>

        <p className="text-center text-xs text-slate-600 mt-6">
          Las credenciales se almacenan de forma segura en el llavero del sistema operativo
        </p>
      </div>
    </div>
  );
}
