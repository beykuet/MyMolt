import { Dashboard } from './pages/Dashboard';
import { PairingScreen } from './components/layout/PairingScreen';
import { useAuth } from './hooks/useAuth';
import { Mascot } from './components/ui/Mascot';

function App() {
  const { isPaired, login, logout, loading } = useAuth();

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-zinc-50 dark:bg-zinc-950">
        <Mascot />
      </div>
    );
  }

  if (!isPaired) {
    return <PairingScreen onPair={async (code) => {
      const res = await login(code);
      return { success: res.success || false, error: res.error };
    }} />;
  }

  return <Dashboard onLogout={logout} />;
}

export default App;
