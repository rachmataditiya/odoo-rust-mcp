import React, { useState, useEffect } from 'react';
import { JsonEditor } from './JsonEditor';
import { StatusMessage } from './StatusMessage';
import { useConfig } from '../hooks/useConfig';
import type { ServerConfig } from '../types';

export const ServerTab: React.FC = () => {
  const { load, save, status, loading } = useConfig('server');
  const [config, setConfig] = useState<ServerConfig>({});

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await load();
      setConfig(data || {});
    } catch (error) {
      console.error('Failed to load server config:', error);
    }
  };

  const handleSave = async () => {
    try {
      await save(config);
    } catch (error) {
      console.error('Failed to save server config:', error);
    }
  };

  const handleRefresh = () => {
    loadConfig();
  };

  return (
    <div>
      <div className="bg-blue-50 border-l-4 border-blue-500 p-3 rounded mb-5 text-blue-800 text-sm">
        ⚙️ Configure server metadata and behavior settings. These affect how the MCP server is presented to clients.
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1fr_400px] gap-8 mb-5">
        <div className="flex flex-col min-w-0">
          <label className="block mb-2 font-medium text-gray-700 text-sm">
            Server Configuration (JSON)
          </label>
          <JsonEditor value={config} onChange={setConfig} />
          <StatusMessage status={status} />
          <div className="flex gap-2 mt-4 flex-wrap">
            <button
              onClick={handleSave}
              disabled={loading}
              className="px-5 py-2 bg-primary text-white rounded cursor-pointer font-medium transition-all hover:bg-primary-dark hover:shadow-lg disabled:opacity-50"
            >
              💾 Save Server Config
            </button>
            <button
              onClick={handleRefresh}
              disabled={loading}
              className="px-5 py-2 bg-gray-100 text-gray-700 rounded cursor-pointer font-medium transition-all hover:bg-gray-200 disabled:opacity-50"
            >
              🔄 Refresh
            </button>
          </div>
        </div>

        <div className="flex flex-col gap-4 sticky top-5 max-h-[calc(100vh-200px)] overflow-y-auto">
          <div className="bg-blue-50 border-l-4 border-blue-500 p-3 rounded text-blue-800 text-sm">
            <strong>Server Configuration Fields:</strong>
            <br />
            • <code className="bg-white px-1.5 py-0.5 rounded text-xs">serverName</code> - Name shown to MCP clients
            <br />
            • <code className="bg-white px-1.5 py-0.5 rounded text-xs">instructions</code> - System instructions for the AI
            <br />
            • <code className="bg-white px-1.5 py-0.5 rounded text-xs">protocolVersionDefault</code> - Default MCP protocol version
          </div>
        </div>
      </div>
    </div>
  );
};
