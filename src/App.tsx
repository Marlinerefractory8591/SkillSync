import React, { useEffect } from 'react';
import { useSkillStore } from './store/useSkillStore';
import { Navbar } from './components/Navbar';
import { SkillList } from './components/SkillList';
import { SkillDetailModal } from './components/SkillDetailModal';
import { SettingsModal } from './components/SettingsModal';
import { UpdateCenterModal } from './components/UpdateCenterModal';
import { DirtyWorktreeConfirmation } from './components/DirtyWorktreeConfirmation';
import { AlertCircle, X } from 'lucide-react';

export const App: React.FC = () => {
  const { fetchSkills, loadConfig, error, theme } = useSkillStore();

  useEffect(() => {
    fetchSkills();
    loadConfig();
  }, [fetchSkills, loadConfig]);

  // Ensure root theme class is synced
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col font-sans">
      {/* Sticky Navbar */}
      <Navbar />

      {/* Main Content Area */}
      <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Error Toast / Banner */}
        {error && (
          <div className="mb-6 p-4 rounded-xl border border-rose-500/30 bg-rose-500/10 flex items-center justify-between gap-3 text-xs text-rose-400">
            <div className="flex items-center gap-2">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span>{error}</span>
            </div>
            <button
              onClick={() => useSkillStore.setState({ error: null })}
              className="hover:text-rose-200 transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        )}

        {/* List of Skills */}
        <SkillList />
      </main>

      {/* Modals */}
      <SkillDetailModal />
      <SettingsModal />
      <UpdateCenterModal />
      <DirtyWorktreeConfirmation />
    </div>
  );
};

export default App;
