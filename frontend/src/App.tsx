import React from 'react';
import { AuthProvider, useAuth } from './context/AuthContext';
import { Lobby } from './pages/Lobby';
import { Dashboard } from './pages/Dashboard';

const AppContent: React.FC = () => {
  const { user } = useAuth();

  if (!user) {
    return <Lobby />;
  }

  return <Dashboard role={user.role} />;
};

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}

export default App;
