import React from 'react';
import { Database, Server, Wrench, FileText, Circle } from 'lucide-react';
import type { TabType } from '../types';
import packageJson from '../../package.json';

interface SideNavProps {
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
  isLive?: boolean;
}

export function SideNav({ activeTab, onTabChange, isLive = true }: SideNavProps) {
  const navItems = [
    { id: 'server' as TabType, label: 'Server', icon: Server },
    { id: 'instances' as TabType, label: 'Instances', icon: Database },
    { id: 'tools' as TabType, label: 'Tools', icon: Wrench },
    { id: 'prompts' as TabType, label: 'Prompts', icon: FileText },
  ];

  return (
    <div className="w-64 h-screen bg-gray-900 text-white flex flex-col">
      <div className="p-6 border-b border-gray-800">
        <h1 className="text-xl font-bold">MCP Configuration</h1>
        <p className="text-sm text-gray-400 mt-1">Manager v{packageJson.version}</p>
      </div>

      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;

          return (
            <button
              key={item.id}
              onClick={() => onTabChange(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all duration-200 ${
                isActive
                  ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/30'
                  : 'text-gray-300 hover:bg-gray-800 hover:text-white'
              }`}
            >
              <Icon size={20} />
              <span className="font-medium">{item.label}</span>
            </button>
          );
        })}
      </nav>

      <div className="p-4 border-t border-gray-800">
        <div className="flex items-center gap-2 px-4 py-2">
          <Circle
            className={`${isLive ? 'text-green-500 fill-green-500' : 'text-gray-500 fill-gray-500'}`}
            size={8}
          />
          <span className="text-sm text-gray-400">
            {isLive ? 'Hot Reload Active' : 'Disconnected'}
          </span>
        </div>
      </div>
    </div>
  );
}
