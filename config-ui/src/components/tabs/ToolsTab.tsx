import React, { useEffect, useState } from 'react';
import { RefreshCw, Search } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Card } from '../Card';
import { Button } from '../Button';
import { StatusMessage } from '../StatusMessage';
import { ToolDetail } from '../ToolDetail';
import type { ToolConfig } from '../../types';

const AVAILABLE_GUARDS = [
  {
    key: 'requiresEnvTrue',
    value: 'ODOO_ENABLE_WRITE_TOOLS',
    label: 'Require Write Tools Enabled',
  },
  {
    key: 'requiresEnvTrue',
    value: 'ODOO_ENABLE_CLEANUP_TOOLS',
    label: 'Require Cleanup Tools Enabled',
  },
];

export function ToolsTab() {
  const { load, save, status, loading } = useConfig('tools');
  const [tools, setTools] = useState<ToolConfig[]>([]);
  const [editedTools, setEditedTools] = useState<ToolConfig[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'enabled' | 'disabled'>('all');

  useEffect(() => {
    loadTools();
  }, []);

  const loadTools = async () => {
    try {
      const data = await load() as ToolConfig[];
      setTools(data);
      setEditedTools(data);
    } catch (error) {
      console.error('Failed to load tools:', error);
    }
  };

  const isToolEnabled = (tool: ToolConfig) => {
    return !tool.guards?.requiresEnvTrue;
  };

  const autoSave = async (updatedTools: ToolConfig[]) => {
    try {
      await save(updatedTools);
      setTools(updatedTools);
      setEditedTools(updatedTools);
    } catch (error) {
      console.error('Failed to auto-save tools:', error);
    }
  };

  const toggleTool = async (toolName: string, enabled: boolean) => {
    const updatedTools = editedTools.map(tool => {
      if (tool.name === toolName) {
        const newTool = { ...tool };
        if (!enabled) {
          newTool.guards = {
            ...newTool.guards,
            requiresEnvTrue: `ENABLE_${toolName.toUpperCase().replace(/[^A-Z0-9]/g, '_')}`
          };
        } else {
          if (newTool.guards?.requiresEnvTrue) {
            const { requiresEnvTrue, ...restGuards } = newTool.guards;
            if (Object.keys(restGuards).length === 0) {
              delete newTool.guards;
            } else {
              newTool.guards = restGuards;
            }
          }
        }
        return newTool;
      }
      return tool;
    });
    setEditedTools(updatedTools);
    await autoSave(updatedTools);
  };

  const toggleAll = async (enabled: boolean) => {
    const updatedTools = editedTools.map(tool => {
      const newTool = { ...tool };
      if (!enabled) {
        newTool.guards = {
          ...newTool.guards,
          requiresEnvTrue: `ENABLE_${tool.name.toUpperCase().replace(/[^A-Z0-9]/g, '_')}`
        };
      } else {
        if (newTool.guards?.requiresEnvTrue) {
          const { requiresEnvTrue, ...restGuards } = newTool.guards;
          if (Object.keys(restGuards).length === 0) {
            delete newTool.guards;
          } else {
            newTool.guards = restGuards;
          }
        }
      }
      return newTool;
    });
    setEditedTools(updatedTools);
    await autoSave(updatedTools);
  };

  const toggleGuard = async (toolName: string, guardKey: string, enabled: boolean) => {
    const updatedTools = editedTools.map(tool => {
      if (tool.name === toolName) {
        const newTool = { ...tool };

        if (enabled) {
          const guard = AVAILABLE_GUARDS.find(g => g.key === guardKey);
          if (guard) {
            newTool.guards = {
              ...newTool.guards,
              [guardKey]: guard.value,
            };
          }
        } else {
          if (newTool.guards) {
            const newGuards = { ...newTool.guards };
            delete newGuards[guardKey];

            if (Object.keys(newGuards).length === 0) {
              delete newTool.guards;
            } else {
              newTool.guards = newGuards;
            }
          }
        }

        return newTool;
      }
      return tool;
    });
    setEditedTools(updatedTools);
    await autoSave(updatedTools);
  };

  const filteredTools = editedTools.filter(tool => {
    const matchesSearch = !searchQuery ||
      tool.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (tool.description?.toLowerCase() || '').includes(searchQuery.toLowerCase());

    const enabled = isToolEnabled(tool);
    const matchesFilter =
      filterType === 'all' ||
      (filterType === 'enabled' && enabled) ||
      (filterType === 'disabled' && !enabled);

    return matchesSearch && matchesFilter;
  });

  const enabledCount = editedTools.filter(t => isToolEnabled(t)).length;
  const disabledCount = editedTools.length - enabledCount;

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900">MCP Tools</h2>
        <p className="mt-2 text-gray-600">
          View and manage available Odoo MCP tools. Toggle individual tools to enable or disable them.
        </p>
      </div>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <Card className="bg-gradient-to-br from-blue-50 to-white border-blue-200">
          <div className="text-center">
            <p className="text-sm text-gray-600 mb-1">Total Tools</p>
            <p className="text-3xl font-bold text-gray-900">{editedTools.length}</p>
          </div>
        </Card>

        <Card className="bg-gradient-to-br from-green-50 to-white border-green-200">
          <div className="text-center">
            <p className="text-sm text-gray-600 mb-1">Enabled</p>
            <p className="text-3xl font-bold text-green-600">{enabledCount}</p>
          </div>
        </Card>

        <Card className="bg-gradient-to-br from-gray-50 to-white border-gray-200">
          <div className="text-center">
            <p className="text-sm text-gray-600 mb-1">Disabled</p>
            <p className="text-3xl font-bold text-gray-500">{disabledCount}</p>
          </div>
        </Card>

        <Card className="bg-gradient-to-br from-orange-50 to-white border-orange-200">
          <div className="text-center">
            <p className="text-sm text-gray-600 mb-1">With Guards</p>
            <p className="text-3xl font-bold text-orange-600">
              {editedTools.filter(t => t.guards && Object.keys(t.guards).length > 0).length}
            </p>
          </div>
        </Card>
      </div>

      <Card>
        <div className="flex flex-col sm:flex-row gap-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
            <input
              type="text"
              placeholder="Search tools by name or description..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>

          <div className="flex gap-2">
            <button
              onClick={() => setFilterType('all')}
              className={`px-4 py-2 rounded-lg font-medium text-sm transition-colors ${
                filterType === 'all'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              All
            </button>
            <button
              onClick={() => setFilterType('enabled')}
              className={`px-4 py-2 rounded-lg font-medium text-sm transition-colors ${
                filterType === 'enabled'
                  ? 'bg-green-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              Enabled
            </button>
            <button
              onClick={() => setFilterType('disabled')}
              className={`px-4 py-2 rounded-lg font-medium text-sm transition-colors ${
                filterType === 'disabled'
                  ? 'bg-gray-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              Disabled
            </button>
          </div>
        </div>
      </Card>

      <div className="flex items-center justify-between">
        <p className="text-sm text-gray-600">
          Showing {filteredTools.length} of {editedTools.length} tools
        </p>
        <div className="flex gap-2">
          <Button
            onClick={() => toggleAll(true)}
            variant="secondary"
            size="sm"
          >
            Enable All
          </Button>
          <Button
            onClick={() => toggleAll(false)}
            variant="secondary"
            size="sm"
          >
            Disable All
          </Button>
          <Button
            onClick={loadTools}
            loading={loading}
            icon={<RefreshCw size={14} />}
            variant="secondary"
            size="sm"
          >
            Refresh
          </Button>
        </div>
      </div>

      <div className="space-y-3">
        {filteredTools.length === 0 ? (
          <Card>
            <div className="text-center py-8">
              <p className="text-gray-500">No tools found matching your criteria</p>
            </div>
          </Card>
        ) : (
          filteredTools.map(tool => (
            <ToolDetail
              key={tool.name}
              tool={tool}
              enabled={isToolEnabled(tool)}
              onToggle={async (enabled) => await toggleTool(tool.name, enabled)}
              onToggleGuard={async (guardKey, enabled) => await toggleGuard(tool.name, guardKey, enabled)}
              availableGuards={AVAILABLE_GUARDS}
            />
          ))
        )}
      </div>
    </div>
  );
}
