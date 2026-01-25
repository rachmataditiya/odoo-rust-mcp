import React from 'react';

interface TabsProps {
  activeTab: string;
  onTabChange: (tab: string) => void;
  children: React.ReactNode;
}

export const Tabs: React.FC<TabsProps> = ({ activeTab, onTabChange, children }) => {
  const tabs = [
    { id: 'instances', label: '🏢 Instances' },
    { id: 'server', label: '⚙️ Server' },
    { id: 'tools', label: '🛠️ Tools' },
    { id: 'prompts', label: '💬 Prompts' },
  ];

  return (
    <div>
      <div className="flex gap-2 mb-5 flex-wrap">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => onTabChange(tab.id)}
            className={`px-5 py-2 border-none rounded cursor-pointer font-medium transition-all ${
              activeTab === tab.id
                ? 'bg-white text-primary'
                : 'bg-white/20 text-white hover:bg-white/30'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>
      {children}
    </div>
  );
};
