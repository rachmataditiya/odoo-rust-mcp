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
    <div className="flex min-h-screen bg-slate-950">
      <SideNav items={navItems} activeTab={activeTab} onTabChange={setActiveTab} />
      
      <main className="flex-1 flex flex-col">
        <header className="bg-slate-900 border-b border-slate-700 px-8 py-6">
          <h2 className="text-2xl font-bold text-slate-100">{getPageTitle()}</h2>
          <p className="text-slate-400 text-sm mt-1">Manage your Odoo MCP server configuration</p>
        </header>

        <div className="flex-1 p-8 overflow-auto">
          <div className="max-w-6xl">
            {renderTabContent()}
          </div>
        </div>

        <footer className="bg-slate-900 border-t border-slate-700 px-8 py-4 text-center text-slate-500 text-xs">
          <p>🔄 Hot reload enabled • Changes apply immediately</p>
        </footer>
      </main>
    </div>
  );
}

export default App;
