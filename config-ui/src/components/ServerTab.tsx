import React, { useState, useEffect } from 'react';
import { JsonEditor } from './JsonEditor';
import { StatusMessage } from './StatusMessage';
import { Card } from './Card';
import { Button } from './Button';
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
    <div className="space-y-6">
      <Card 
        title="Server Configuration"
        description="Configure server metadata and behavior settings. These affect how the MCP server is presented to clients."
      >
        <div className="space-y-4">
          <div>
            <label className="block mb-3 font-medium text-slate-200 text-sm">
              Configuration (JSON)
            </label>
            <JsonEditor value={config} onChange={setConfig} />
          </div>
          
          <StatusMessage status={status} />
          
          <div className="flex gap-3 flex-wrap pt-4">
            <Button
              onClick={handleSave}
              disabled={loading}
              icon="💾"
            >
              Save Configuration
            </Button>
            <Button
              onClick={handleRefresh}
              disabled={loading}
              variant="secondary"
              icon="🔄"
            >
              Refresh
            </Button>
          </div>
        </div>
      </Card>

      <Card 
        title="Configuration Reference"
        description="Common server configuration fields"
      >
        <div className="space-y-3 text-sm">
          <div className="space-y-1">
            <p className="text-slate-300">
              <span className="bg-slate-700 px-2 py-1 rounded text-blue-300 font-mono text-xs">serverName</span>
            </p>
            <p className="text-slate-400">Name shown to MCP clients</p>
          </div>
          <div className="space-y-1">
            <p className="text-slate-300">
              <span className="bg-slate-700 px-2 py-1 rounded text-blue-300 font-mono text-xs">instructions</span>
            </p>
            <p className="text-slate-400">System instructions for the AI</p>
          </div>
          <div className="space-y-1">
            <p className="text-slate-300">
              <span className="bg-slate-700 px-2 py-1 rounded text-blue-300 font-mono text-xs">protocolVersionDefault</span>
            </p>
            <p className="text-slate-400">Default MCP protocol version</p>
          </div>
        </div>
      </Card>
    </div>
  );
};
