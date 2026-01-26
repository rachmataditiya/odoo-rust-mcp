import React, { useState, useEffect } from 'react';
import { JsonEditor } from './JsonEditor';
import { StatusMessage } from './StatusMessage';
import { Card } from './Card';
import { Button } from './Button';
import { useConfig } from '../hooks/useConfig';
import type { PromptConfig } from '../types';

export const PromptsTab: React.FC = () => {
  const { load, save, status, loading } = useConfig('prompts');
  const [config, setConfig] = useState<PromptConfig[]>([]);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await load();
      setConfig(Array.isArray(data) ? data : []);
    } catch (error) {
      console.error('Failed to load prompts:', error);
    }
  };

  const handleSave = async () => {
    try {
      await save(config);
    } catch (error) {
      console.error('Failed to save prompts:', error);
    }
  };

  const handleRefresh = () => {
    loadConfig();
  };

  return (
    <div className="space-y-6">
      <Card 
        title="Prompts Configuration"
        description="Define system prompts that guide the MCP server's behavior and responses."
      >
        <div className="space-y-4">
          <div>
            <label className="block mb-3 font-medium text-slate-200 text-sm">
              Configuration (JSON Array)
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
              Save Prompts
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

      {config.length > 0 && (
        <Card 
          title="Configured Prompts"
          description={`${config.length} prompt${config.length !== 1 ? 's' : ''} configured`}
        >
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {config.map((prompt, index) => (
              <div
                key={index}
                className="bg-slate-700 p-4 rounded-lg border border-slate-600 hover:border-blue-500 transition-colors"
              >
                <h4 className="font-semibold text-slate-100 mb-1">{prompt.name}</h4>
                {prompt.description && (
                  <p className="text-slate-400 text-sm mb-2">{prompt.description}</p>
                )}
                {prompt.content && (
                  <p className="text-slate-500 text-xs line-clamp-2">{prompt.content.substring(0, 100)}</p>
                )}
              </div>
            ))}
          </div>
        </Card>
      )}

      <Card 
        title="Configuration Reference"
        description="Common prompt configuration fields"
      >
        <div className="space-y-3 text-sm">
          <div className="space-y-1">
            <p className="text-slate-300">
              <span className="bg-slate-700 px-2 py-1 rounded text-blue-300 font-mono text-xs">name</span>
            </p>
            <p className="text-slate-400">Unique prompt identifier</p>
          </div>
          <div className="space-y-1">
            <p className="text-slate-300">
              <span className="bg-slate-700 px-2 py-1 rounded text-blue-300 font-mono text-xs">description</span>
            </p>
            <p className="text-slate-400">What this prompt does</p>
          </div>
          <div className="space-y-1">
            <p className="text-slate-300">
              <span className="bg-slate-700 px-2 py-1 rounded text-blue-300 font-mono text-xs">content</span>
            </p>
            <p className="text-slate-400">The actual prompt text</p>
          </div>
        </div>
      </Card>
    </div>
  );
};
