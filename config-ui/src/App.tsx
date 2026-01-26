import { useState } from 'react';
import { SideNav } from './components/SideNav';
import { InstancesTab } from './components/InstancesTab';
import { ServerTab } from './components/ServerTab';
import { ToolsTab } from './components/ToolsTab';
import { PromptsTab } from './components/PromptsTab';

function App() {
  const [activeTab, setActiveTab] = useState('instances');

  const navItems = [
    { id: 'instances', label: 'Instances', icon: '🏢' },
    { id: 'server', label: 'Server', icon: '⚙️' },
    { id: 'tools', label: 'Tools', icon: '🛠️' },
    { id: 'prompts', label: 'Prompts', icon: '💬' },
  ];

  const renderTabContent = () => {
    switch (activeTab) {
      case 'instances':
        return <InstancesTab />;
      case 'server':
        return <ServerTab />;
      case 'tools':
        return <ToolsTab />;
      case 'prompts':
        return <PromptsTab />;
      default:
        return <InstancesTab />;
    }
  };

  const getPageTitle = () => {
    const item = navItems.find(i => i.id === activeTab);
    return item ? `${item.icon} ${item.label}` : 'Config Manager';
  };

  return (
    <div className="flex min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      <SideNav items={navItems} activeTab={activeTab} onTabChange={setActiveTab} />
      
      <main className="flex-1 flex flex-col overflow-hidden">
        <header className="bg-gradient-to-r from-slate-900 to-slate-800 border-b border-slate-700 px-8 py-6 shadow-lg">
          <div className="max-w-7xl">
            <h2 className="text-3xl font-bold bg-gradient-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent">{getPageTitle()}</h2>
            <p className="text-slate-400 text-sm mt-2">Manage your Odoo MCP server configuration in real-time</p>
          </div>
        </header>

        <div className="flex-1 overflow-auto">
          <div className="p-8 max-w-7xl mx-auto">
            {renderTabContent()}
          </div>
        </div>

        <footer className="bg-slate-950 border-t border-slate-700 px-8 py-4">
          <div className="max-w-7xl mx-auto">
            <p className="text-center text-slate-500 text-xs">
              <span className="inline-block mr-2">🔄</span>
              Hot reload enabled • Changes apply immediately
              <span className="inline-block ml-2">v0.3.15</span>
            </p>
          </div>
        </footer>
      </main>
    </div>
  );
}

export default App;
