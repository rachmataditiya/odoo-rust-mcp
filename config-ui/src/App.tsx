import { useState } from 'react';
import { Tabs } from './components/Tabs';
import { InstancesTab } from './components/InstancesTab';
import { ServerTab } from './components/ServerTab';
import { ToolsTab } from './components/ToolsTab';
import { PromptsTab } from './components/PromptsTab';

function App() {
  const [activeTab, setActiveTab] = useState('instances');

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

  return (
    <div className="min-h-screen bg-gradient-to-br from-primary to-secondary p-5">
      <div className="max-w-[1600px] mx-auto">
        <header className="bg-white/95 p-8 rounded-lg mb-8 shadow-lg">
          <h1 className="text-primary text-3xl mb-2 font-bold">
            🚀 Odoo Rust MCP Config Manager
          </h1>
          <p className="text-gray-600 text-sm">
            Edit and manage your Odoo MCP server configuration in real-time
          </p>
        </header>

        <Tabs activeTab={activeTab} onTabChange={setActiveTab}>
          <div className="bg-white p-8 rounded-lg shadow-lg">
            {renderTabContent()}
          </div>
        </Tabs>

        <footer className="text-center text-white/80 mt-8 text-sm">
          <p>🔄 Hot reload enabled • Changes apply immediately to the running server</p>
          <p className="mt-2 opacity-70">
            Odoo Rust MCP Server • Config UI • Port 3008 (Inspired by Peugeot 3008)
          </p>
        </footer>
      </div>
    </div>
  );
}

export default App;
