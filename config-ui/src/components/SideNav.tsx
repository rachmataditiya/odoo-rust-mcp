import {
  Database,
  Server,
  Wrench,
  FileText,
  Shield,
  LogOut,
  Sun,
  Moon,
  Laptop,
} from 'lucide-react';
import type { TabType } from '../types';
import { useAuth } from './AuthProvider';
import { useTheme } from '../hooks/useTheme';
import { Brand } from './Brand';

export interface SideNavProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
  isLive?: boolean;
  onNavigate?: () => void;
}

export function SideNav({
  activeTab,
  onTabChange,
  isLive = true,
  onNavigate,
}: SideNavProps) {
  const { username, authEnabled, logout } = useAuth();
  const { theme, setTheme } = useTheme();

  const navItems = [
    { id: 'server' as TabType, label: 'Server', icon: Server, description: 'Core & settings' },
    { id: 'instances' as TabType, label: 'Instances', icon: Database, description: 'Odoo connections' },
    { id: 'tools' as TabType, label: 'Tools', icon: Wrench, description: 'Operations & guards' },
    { id: 'prompts' as TabType, label: 'Prompts', icon: FileText, description: 'System prompts' },
    { id: 'security' as TabType, label: 'Security', icon: Shield, description: 'Auth & tokens' },
  ];

  const handleSelect = (id: TabType) => {
    onTabChange(id);
    onNavigate?.();
  };

  return (
    <div className="flex h-full flex-col bg-surface border-r border-line select-none">
      {/* Brand Header */}
      <div className="px-5 pt-6 pb-5 border-b border-line/60">
        <Brand />
      </div>

      {/* Navigation Links */}
      <nav aria-label="Main navigation" className="flex-1 overflow-y-auto px-3 py-4 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;

          return (
            <button
              key={item.id}
              type="button"
              onClick={() => handleSelect(item.id)}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left text-[13px] font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                isActive
                  ? 'bg-primary text-white shadow-sm'
                  : 'text-muted hover:bg-surface-2 hover:text-fg'
              }`}
            >
              <Icon size={16} className={`shrink-0 ${isActive ? 'text-white' : 'text-muted'}`} />
              <div className="min-w-0 flex-1">
                <div className="truncate">{item.label}</div>
              </div>
            </button>
          );
        })}
      </nav>

      {/* Footer Info & Controls */}
      <div className="p-3 border-t border-line bg-surface-2/30 space-y-2.5">
        {/* Hot Reload Status Pill */}
        <div className="flex items-center justify-between px-2.5 py-1.5 rounded-lg border border-line bg-surface text-[12px]">
          <div className="flex items-center gap-2">
            <span className="relative flex h-2 w-2">
              {isLive && (
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-ok opacity-75" />
              )}
              <span
                className={`relative inline-flex rounded-full h-2 w-2 ${
                  isLive ? 'bg-ok' : 'bg-faint'
                }`}
              />
            </span>
            <span className="text-muted font-medium">
              {isLive ? 'Hot reload active' : 'Offline'}
            </span>
          </div>

          {/* Theme switcher */}
          <div className="flex items-center gap-0.5 p-0.5 rounded-md border border-line bg-surface-2">
            <button
              type="button"
              onClick={() => setTheme('light')}
              title="Light theme"
              aria-label="Light theme"
              className={`p-1 rounded transition-colors ${
                theme === 'light'
                  ? 'bg-surface text-fg shadow-xs'
                  : 'text-muted hover:text-fg'
              }`}
            >
              <Sun size={12} />
            </button>
            <button
              type="button"
              onClick={() => setTheme('auto')}
              title="System theme"
              aria-label="System theme"
              className={`p-1 rounded transition-colors ${
                theme === 'auto'
                  ? 'bg-surface text-fg shadow-xs'
                  : 'text-muted hover:text-fg'
              }`}
            >
              <Laptop size={12} />
            </button>
            <button
              type="button"
              onClick={() => setTheme('dark')}
              title="Dark theme"
              aria-label="Dark theme"
              className={`p-1 rounded transition-colors ${
                theme === 'dark'
                  ? 'bg-surface text-fg shadow-xs'
                  : 'text-muted hover:text-fg'
              }`}
            >
              <Moon size={12} />
            </button>
          </div>
        </div>

        {/* User Account / Sign out */}
        {authEnabled && username && (
          <div className="flex items-center justify-between px-2.5 py-1.5 rounded-lg border border-line bg-surface text-[12px]">
            <div className="min-w-0 flex items-center gap-2">
              <div className="w-5 h-5 rounded-full bg-primary/10 text-primary text-[10px] font-semibold flex items-center justify-center shrink-0">
                {username.charAt(0).toUpperCase()}
              </div>
              <span className="font-medium text-fg truncate" title={username}>
                {username}
              </span>
            </div>
            <button
              type="button"
              onClick={logout}
              className="p-1 text-muted hover:text-down rounded transition-colors"
              title="Sign out"
              aria-label="Sign out"
            >
              <LogOut size={13} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
