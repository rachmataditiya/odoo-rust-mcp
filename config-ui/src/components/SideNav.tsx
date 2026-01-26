import React from 'react';

interface NavItem {
  id: string;
  label: string;
  icon: string;
}

interface SideNavProps {
  items: NavItem[];
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export const SideNav: React.FC<SideNavProps> = ({ items, activeTab, onTabChange }) => {
  return (
    <nav className="w-64 bg-slate-900 border-r border-slate-700 p-6 min-h-screen flex flex-col">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-blue-400">🚀 MCP Config</h1>
        <p className="text-xs text-slate-400 mt-1">Odoo Rust Manager</p>
      </div>

      <div className="space-y-2 flex-1">
        {items.map((item) => (
          <button
            key={item.id}
            onClick={() => onTabChange(item.id)}
            className={`w-full text-left px-4 py-3 rounded-lg transition-all font-medium flex items-center gap-3 ${
              activeTab === item.id
                ? 'bg-blue-600 text-white'
                : 'text-slate-300 hover:bg-slate-800'
            }`}
          >
            <span className="text-lg">{item.icon}</span>
            {item.label}
          </button>
        ))}
      </div>

      <div className="pt-4 border-t border-slate-700">
        <p className="text-xs text-slate-500 text-center">
          🔄 Hot Reload Enabled
        </p>
      </div>
    </nav>
  );
};
