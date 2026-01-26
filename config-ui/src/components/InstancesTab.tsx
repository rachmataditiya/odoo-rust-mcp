import React, { useState, useEffect } from 'react';
import { JsonEditor } from './JsonEditor';
import { StatusMessage } from './StatusMessage';
import { Card } from './Card';
import { Button } from './Button';
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
    <div className="space-y-6">
      <Card 
        title="Instances Configuration"
        description="Configure Odoo instances that this MCP server can connect to. Changes are applied immediately."
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
              Save Instances
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

      {Object.keys(instances).length > 0 && (
        <Card 
          title="Active Instances"
          description={`${Object.keys(instances).length} instance${Object.keys(instances).length !== 1 ? 's' : ''} configured`}
        >
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Object.entries(instances).map(([name, instanceConfig]) => (
              <div
                key={name}
                className="bg-gradient-to-br from-slate-700/50 to-slate-800/50 p-4 rounded-lg border border-slate-700 hover:border-blue-500 hover:shadow-lg hover:shadow-blue-900/20 transition-all duration-200"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <h4 className="font-bold text-slate-100 mb-1">{name}</h4>
                    <p className="text-slate-400 font-mono text-xs break-all line-clamp-2 hover:line-clamp-none transition-all">
                      {instanceConfig.url || 'No URL configured'}
                    </p>
                  </div>
                  <span className="text-xl ml-3 flex-shrink-0">🏢</span>
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
};
