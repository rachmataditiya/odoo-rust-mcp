import { useEffect, useState } from 'react';
import { RefreshCw, Search, CheckCheck, XCircle } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Button } from '../Button';
import { TextField } from '../TextField';
import { Segmented } from '../Segmented';
import { StatusMessage } from '../StatusMessage';
import { ToolDetail } from '../ToolDetail';
import { PageHeader } from '../PageHeader';
import type { ToolConfig } from '../../types';

export function ToolsTab() {
  const { load, save, status, loading } = useConfig('tools');
  const [editedTools, setEditedTools] = useState<ToolConfig[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<'all' | 'enabled' | 'disabled'>('all');

  useEffect(() => {
    loadTools();
  }, []);

  const loadTools = async () => {
    try {
      const data = (await load()) as ToolConfig[];
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
      setEditedTools(updatedTools);
    } catch (error) {
      console.error('Failed to auto-save tools:', error);
    }
  };

  const toggleTool = async (toolName: string, enabled: boolean) => {
    const updatedTools = editedTools.map((tool) => {
      if (tool.name === toolName) {
        const newTool = { ...tool };
        if (!enabled) {
          newTool.guards = {
            ...newTool.guards,
            requiresEnvTrue: `ENABLE_${toolName.toUpperCase().replace(/[^A-Z0-9]/g, '_')}`,
          };
        } else {
          if (newTool.guards?.requiresEnvTrue) {
            const { requiresEnvTrue: _, ...restGuards } = newTool.guards;
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
    const updatedTools = editedTools.map((tool) => {
      const newTool = { ...tool };
      if (!enabled) {
        newTool.guards = {
          ...newTool.guards,
          requiresEnvTrue: `ENABLE_${tool.name.toUpperCase().replace(/[^A-Z0-9]/g, '_')}`,
        };
      } else {
        if (newTool.guards?.requiresEnvTrue) {
          const { requiresEnvTrue: _, ...restGuards } = newTool.guards;
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

  const filteredTools = editedTools.filter((tool) => {
    const matchesSearch =
      tool.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (tool.description && tool.description.toLowerCase().includes(searchQuery.toLowerCase()));

    if (!matchesSearch) return false;

    const enabled = isToolEnabled(tool);
    if (filterType === 'enabled') return enabled;
    if (filterType === 'disabled') return !enabled;
    return true;
  });

  const enabledCount = editedTools.filter(isToolEnabled).length;
  const disabledCount = editedTools.length - enabledCount;

  return (
    <div className="space-y-6">
      <PageHeader
        title="MCP Tools"
        description="Enable or disable specific Odoo operations exposed to AI assistants."
      >
        <Button
          onClick={() => toggleAll(true)}
          variant="plain"
          size="sm"
          icon={<CheckCheck size={14} className="text-ok" />}
          disabled={loading || enabledCount === editedTools.length}
        >
          Enable All
        </Button>
        <Button
          onClick={() => toggleAll(false)}
          variant="plain"
          size="sm"
          icon={<XCircle size={14} className="text-down" />}
          disabled={loading || disabledCount === editedTools.length}
        >
          Disable All
        </Button>
        <Button
          onClick={loadTools}
          variant="plain"
          size="sm"
          icon={<RefreshCw size={13} className={loading ? 'animate-spin' : ''} />}
          disabled={loading}
        >
          Reload
        </Button>
      </PageHeader>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      {/* Search and Filters Bar */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
        <div className="flex-1 sm:max-w-xs">
          <TextField
            placeholder="Search tools by name or description..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            icon={<Search size={14} />}
          />
        </div>

        <Segmented<'all' | 'enabled' | 'disabled'>
          options={[
            { label: `All (${editedTools.length})`, value: 'all' },
            { label: `Enabled (${enabledCount})`, value: 'enabled' },
            { label: `Disabled (${disabledCount})`, value: 'disabled' },
          ]}
          value={filterType}
          onChange={(val) => setFilterType(val)}
          className="sm:w-80"
        />
      </div>

      {/* Tools List */}
      <div className="space-y-2.5">
        {filteredTools.length === 0 ? (
          <div className="rounded-xl border border-dashed border-line bg-surface/50 p-10 text-center text-muted text-[13px]">
            No tools found matching your current filter criteria.
          </div>
        ) : (
          filteredTools.map((tool) => (
            <ToolDetail
              key={tool.name}
              tool={tool}
              enabled={isToolEnabled(tool)}
              onToggle={(enabled) => toggleTool(tool.name, enabled)}
            />
          ))
        )}
      </div>
    </div>
  );
}
