import React, { useState, useEffect } from 'react';
import { JsonEditor } from './JsonEditor';
import { StatusMessage } from './StatusMessage';
import { useConfig } from '../hooks/useConfig';
import type { ToolConfig, ToolCategory } from '../types';

const TOOL_CATEGORIES: Record<string, ToolCategory> = {
  'ODOO_ENABLE_WRITE_TOOLS': {
    name: 'Write Operations',
    description: 'Create, update, delete, and modify records',
    icon: '✏️',
    color: '#ef4444',
    bgColor: '#fef2f2',
    tools: ['odoo_create', 'odoo_update', 'odoo_delete', 'odoo_execute', 'odoo_workflow_action', 'odoo_copy', 'odoo_create_batch'],
    envVar: 'ODOO_ENABLE_WRITE_TOOLS',
  },
  'ODOO_ENABLE_CLEANUP_TOOLS': {
    name: 'Destructive Cleanup',
    description: 'Database cleanup and deep cleanup operations',
    icon: '🗑️',
    color: '#dc2626',
    bgColor: '#fee2e2',
    tools: ['odoo_database_cleanup', 'odoo_deep_cleanup'],
    envVar: 'ODOO_ENABLE_CLEANUP_TOOLS',
  },
};

export const ToolsTab: React.FC = () => {
  const { load, save, status, loading } = useConfig('tools');
  const [config, setConfig] = useState<ToolConfig[]>([]);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await load();
      setConfig(Array.isArray(data) ? data : []);
    } catch (error) {
      console.error('Failed to load tools:', error);
    }
  };

  const handleSave = async () => {
    try {
      await save(config);
      await loadConfig(); // Reload to update UI
    } catch (error) {
      console.error('Failed to save tools:', error);
    }
  };

  const handleRefresh = () => {
    loadConfig();
  };

  const updateToolsGuards = async (envVar: string, enabled: boolean) => {
    const updatedConfig = [...config];
    
    const category = TOOL_CATEGORIES[envVar];
    if (!category) return;

    updatedConfig.forEach((tool) => {
      if (category.tools.includes(tool.name)) {
        if (enabled) {
          tool.guards = tool.guards || {};
          tool.guards.requiresEnvTrue = envVar;
        } else {
          if (tool.guards) {
            delete tool.guards.requiresEnvTrue;
            if (Object.keys(tool.guards).length === 0) {
              delete tool.guards;
            }
          }
        }
      }
    });

    setConfig(updatedConfig);
    
    try {
      await save(updatedConfig);
      await loadConfig(); // Reload to update UI
    } catch (error) {
      console.error('Failed to update tool guards:', error);
    }
  };

  const getCategoryStatus = (envVar: string): boolean => {
    const category = TOOL_CATEGORIES[envVar];
    if (!category) return false;

    const categoryTools = config.filter((tool) => category.tools.includes(tool.name));
    return categoryTools.length > 0 && categoryTools.some((tool) => {
      return tool.guards && tool.guards.requiresEnvTrue === envVar;
    });
  };

  const getAllCategoryTools = (): Set<string> => {
    const allTools = new Set<string>();
    Object.values(TOOL_CATEGORIES).forEach((cat) => {
      cat.tools.forEach((toolName) => allTools.add(toolName));
    });
    return allTools;
  };

  const getReadOnlyTools = (): ToolConfig[] => {
    const allCategoryTools = getAllCategoryTools();
    return config.filter((tool) => {
      const hasGuard = tool.guards && tool.guards.requiresEnvTrue;
      const isInCategory = allCategoryTools.has(tool.name);
      return !hasGuard && !isInCategory;
    });
  };

  return (
    <div>
      <div className="bg-blue-50 border-l-4 border-blue-500 p-3 rounded mb-5 text-blue-800 text-sm">
        🛠️ Define available tools that clients can call. Enable/disable tools by category for better security control.
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[1fr_400px] gap-8 mb-5">
        <div className="flex flex-col min-w-0">
          <label className="block mb-2 font-medium text-gray-700 text-sm">
            Tools Configuration (JSON Array)
          </label>
          <JsonEditor value={config} onChange={setConfig} />
          <StatusMessage status={status} />
          <div className="flex gap-2 mt-4 flex-wrap">
            <button
              onClick={handleSave}
              disabled={loading}
              className="px-5 py-2 bg-primary text-white rounded cursor-pointer font-medium transition-all hover:bg-primary-dark hover:shadow-lg disabled:opacity-50"
            >
              💾 Save Tools
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
          {Object.entries(TOOL_CATEGORIES).map(([envVar, category]) => {
            const isEnabled = getCategoryStatus(envVar);
            const categoryTools = config.filter((tool) => category.tools.includes(tool.name));

            return (
              <div
                key={envVar}
                className="p-4 rounded border-2 transition-all"
                style={{
                  background: category.bgColor,
                  borderColor: isEnabled ? category.color : '#e5e7eb',
                }}
              >
                <div className="flex items-start gap-3 mb-2">
                  <span className="text-xl">{category.icon}</span>
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <label
                        htmlFor={`tool-flag-${envVar}`}
                        className="cursor-pointer font-semibold text-gray-700 text-sm"
                      >
                        {category.name}
                      </label>
                      <span
                        className={`px-2 py-0.5 rounded text-xs font-medium text-white ${
                          isEnabled ? 'bg-red-600' : 'bg-gray-400'
                        }`}
                      >
                        {isEnabled ? 'ENABLED' : 'DISABLED'}
                      </span>
                    </div>
                    <div className="text-gray-600 text-xs mb-2">{category.description}</div>
                  </div>
                  <input
                    type="checkbox"
                    id={`tool-flag-${envVar}`}
                    checked={isEnabled}
                    onChange={(e) => updateToolsGuards(envVar, e.target.checked)}
                    className="mt-1 w-5 h-5 cursor-pointer accent-red-600"
                  />
                </div>
                <div className="text-gray-600 text-xs mt-2">
                  <strong>Environment Variable:</strong>{' '}
                  <code className="bg-white px-1.5 py-0.5 rounded text-xs">{envVar}</code>
                </div>
                <div className="text-gray-600 text-xs mt-2">
                  <strong>Tools ({categoryTools.length}):</strong>{' '}
                  <span className="text-gray-700">
                    {categoryTools.map((t) => t.name).join(', ')}
                  </span>
                </div>
              </div>
            );
          })}

          {getReadOnlyTools().length > 0 && (
            <div className="p-4 bg-blue-50 rounded border-2 border-blue-200">
              <div className="flex items-start gap-3">
                <span className="text-xl">📖</span>
                <div className="flex-1">
                  <div className="font-semibold text-gray-700 mb-1 text-sm">
                    Read-Only Tools (Always Enabled)
                  </div>
                  <div className="text-gray-600 text-xs mb-2">
                    These tools are safe and always available. They only read data, never modify.
                  </div>
                  <div className="text-gray-600 text-xs">
                    <strong>Tools ({getReadOnlyTools().length}):</strong>{' '}
                    <span className="text-gray-700">
                      {getReadOnlyTools().map((t) => t.name).join(', ')}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
