import { VaultUnlock } from "./components/vault/VaultUnlock";
import { Dashboard } from "./components/dashboard/MetricsPanel";
import { useVault } from "./hooks/useVault";

function App() {
  const { isUnlocked, isLoading } = useVault();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-surface-900">
        <div className="text-emerald-400 text-lg animate-pulse">
          Inicializando bóveda...
        </div>
      </div>
    );
  }

  if (!isUnlocked) {
    return <VaultUnlock />;
  }

  return <Dashboard />;
}

export default App;
