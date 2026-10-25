import { useState, useEffect } from 'react';
import { AuthProvider, useAuth } from './components/AuthProvider';
import { LoginForm } from './components/LoginForm';
import { SideNav } from './components/SideNav';
import { InstancesTab } from './components/tabs/InstancesTab';
import { ServerTab } from './components/tabs/ServerTab';
import { ToolsTab } from './components/tabs/ToolsTab';
import { PromptsTab } from './components/tabs/PromptsTab';
import { SecurityTab } from './components/tabs/SecurityTab';
import { Brand } from './components/Brand';
import { Loader2, Menu } from 'lucide-react';
import type { TabType } from './types';

function AppContent() {
  const { isAuthenticated, loading } = useAuth();
  const [activeTab, setActiveTab] = useState<TabType>('server');
  const [drawerOpen, setDrawerOpen] = useState(false);

  // Close drawer on Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setDrawerOpen(false);
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Show loading spinner while checking auth
  if (loading) {
    return (
      <div className="min-h-screen bg-base flex flex-col items-center justify-center gap-3">
        <Loader2 className="w-7 h-7 text-primary animate-spin" />
        <span className="text-[13px] font-medium text-muted">
          Loading configuration...
        </span>
      </div>
    );
  }

  // Show login form if not authenticated
  if (!isAuthenticated) {
    return <LoginForm />;
  }

  const renderTab = () => {
    switch (activeTab) {
      case 'instances':
        return <InstancesTab />;
      case 'server':
        return <ServerTab />;
      case 'tools':
        return <ToolsTab />;
      case 'prompts':
        return <PromptsTab />;
      case 'security':
        return <SecurityTab />;
      default:
        return <ServerTab />;
    }
  };

  return (
    <div className="flex min-h-screen bg-base text-fg">
      {/* Desktop Sticky Sidebar */}
      <aside className="sticky top-0 hidden h-screen w-60 shrink-0 md:block">
        <SideNav activeTab={activeTab} onTabChange={setActiveTab} isLive={true} />
      </aside>

      {/* Mobile Drawer */}
      {drawerOpen && (
        <div className="fixed inset-0 z-40 md:hidden">
          <div
            className="fixed inset-0 bg-black/40 backdrop-blur-xs transition-opacity"
            onClick={() => setDrawerOpen(false)}
            aria-hidden="true"
          />
          <aside className="fixed inset-y-0 left-0 w-64 max-w-[85%] bg-surface shadow-2xl z-50 animate-in slide-in-from-left duration-200">
            <SideNav
              activeTab={activeTab}
              onTabChange={setActiveTab}
              isLive={true}
              onNavigate={() => setDrawerOpen(false)}
            />
          </aside>
        </div>
      )}

      {/* Main Content Area */}
      <div className="flex min-w-0 flex-1 flex-col">
        {/* Mobile Header Bar */}
        <div className="sticky top-0 z-30 flex h-14 items-center justify-between border-b border-line bg-surface/80 backdrop-blur-md px-4 md:hidden">
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setDrawerOpen(true)}
              aria-label="Open menu"
              className="grid h-9 w-9 place-items-center rounded-md text-muted transition-colors hover:bg-surface-2 hover:text-fg focus-visible:outline-2 focus-visible:outline-primary"
            >
              <Menu size={18} />
            </button>
            <Brand small />
          </div>
        </div>

        {/* Page Content */}
        <main className="mx-auto w-full max-w-6xl px-4 py-8 sm:px-8 flex-1">
          {renderTab()}
        </main>
      </div>
    </div>
  );
}

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}

export default App;
