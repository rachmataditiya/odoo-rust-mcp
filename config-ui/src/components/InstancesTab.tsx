import React, { useState, useEffect } from 'react';
import { JsonEditor } from './JsonEditor';
import { StatusMessage } from './StatusMessage';
import { useConfig } from '../hooks/useConfig';
import type { InstanceConfig } from '../types';

export const InstancesTab: React.FC = () => {
  const { load, save, status, loading } = useConfig('instances');
  const [config, setConfig] = useState<InstanceConfig>({});
  const [instances, setInstances] = useState<InstanceConfig>({});

  useEffect(() => {
    loadConfig();
    loadInstances();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await load();
      setConfig(data || {});
    } catch (error) {
      console.error('Failed to load instances:', error);
    }
  };

  const loadInstances = async () => {
    try {
      const response = await fetch('/api/config/instances');
      if (response.ok) {
        const data = await response.json();
        setInstances(data || {});
      }
    } catch (error) {
      console.error('Failed to load instances list:', error);
    }
  };

  const handleSave = async () => {
    try {
      await save(config);
      await loadInstances();
    } catch (error) {
      console.error('Failed to save instances:', error);
    }
  };

  const handleRefresh = () => {
    loadConfig();
    loadInstances();
  };

  return (
    <div>
      <div className="bg-blue-50 border-l-4 border-blue-500 p-3 rounded mb-5 text-blue-800 text-sm">
        📌 Configure Odoo instances that this MCP server can connect to. Changes are applied immediately.
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1fr_400px] gap-8 mb-5">
        <div className="flex flex-col min-w-0">
          <label className="block mb-2 font-medium text-gray-700 text-sm">
            Instances Configuration (JSON)
          </label>
          <JsonEditor value={config} onChange={setConfig} />
          <StatusMessage status={status} />
          <div className="flex gap-2 mt-4 flex-wrap">
            <button
              onClick={handleSave}
              disabled={loading}
              className="px-5 py-2 bg-primary text-white rounded cursor-pointer font-medium transition-all hover:bg-primary-dark hover:shadow-lg disabled:opacity-50"
            >
              💾 Save Instances
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
          {Object.keys(instances).length > 0 && (
            <div className="bg-gray-50 rounded p-4 mb-5 max-h-[300px] overflow-y-auto">
              <h3 className="mb-2 text-gray-700 font-semibold">Active Instances</h3>
              <div className="space-y-2">
                {Object.entries(instances).map(([name, instanceConfig]) => (
                  <div
                    key={name}
                    className="bg-white p-3 rounded border-l-4 border-primary flex justify-between items-center text-sm"
                  >
                    <div className="flex-1">
                      <div className="font-semibold text-gray-700 mb-1">{name}</div>
                      <div className="text-gray-600 font-mono text-xs">
                        {instanceConfig.url || 'N/A'}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
