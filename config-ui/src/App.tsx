import React, { useState } from 'react';
import { SideNav } from './components/SideNav';
import { InstancesTab } from './components/tabs/InstancesTab';
import { ServerTab } from './components/tabs/ServerTab';
import { ToolsTab } from './components/tabs/ToolsTab';
import { PromptsTab } from './components/tabs/PromptsTab';
import type { TabType } from './types';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('server');

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
      default:
        return <ServerTab />;
    }
  };

  return (
    <div className="flex h-screen bg-gray-50">
      <SideNav activeTab={activeTab} onTabChange={setActiveTab} isLive={true} />
      <main className="flex-1 overflow-auto">
        <div className="max-w-7xl mx-auto p-8">
          {renderTab()}
        </div>
      </main>
    </div>
  );
}

export default App;
