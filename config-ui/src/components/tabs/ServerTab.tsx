import React, { useEffect, useState } from 'react';
import { Save, RefreshCw, Info, Plus, Trash2 } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Card } from '../Card';
import { Button } from '../Button';
import { StatusMessage } from '../StatusMessage';
import type { ServerConfig } from '../../types';

export function ServerTab() {
  const { load, save, status, loading } = useConfig('server');
  const [serverName, setServerName] = useState('');
  const [instructions, setInstructions] = useState('');
  const [protocolVersionDefault, setProtocolVersionDefault] = useState('');
  const [customFields, setCustomFields] = useState<{ key: string; value: string }[]>([]);

  useEffect(() => {
    loadServer();
  }, []);

  const loadServer = async () => {
    try {
      const data = await load() as ServerConfig;
      setServerName(data.serverName || '');
      setInstructions(data.instructions || '');
      setProtocolVersionDefault(data.protocolVersionDefault || '');

      const customEntries = Object.entries(data)
        .filter(([key]) => !['serverName', 'instructions', 'protocolVersionDefault'].includes(key))
        .map(([key, value]) => ({ key, value: String(value) }));
      setCustomFields(customEntries);
    } catch (error) {
      console.error('Failed to load server config:', error);
    }
  };

  const handleSave = async () => {
    try {
      const config: ServerConfig = {};

      if (serverName.trim()) config.serverName = serverName.trim();
      if (instructions.trim()) config.instructions = instructions.trim();
      if (protocolVersionDefault.trim()) config.protocolVersionDefault = protocolVersionDefault.trim();

      customFields.forEach(({ key, value }) => {
        if (key.trim()) {
          config[key.trim()] = value;
        }
      });

      await save(config);
      await loadServer();
    } catch (error) {
      console.error('Failed to save server config:', error);
    }
  };

  const addCustomField = () => {
    setCustomFields([...customFields, { key: '', value: '' }]);
  };

  const removeCustomField = (index: number) => {
    setCustomFields(customFields.filter((_, i) => i !== index));
  };

  const updateCustomField = (index: number, field: 'key' | 'value', newValue: string) => {
    const updated = [...customFields];
    updated[index][field] = newValue;
    setCustomFields(updated);
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900">Server Configuration</h2>
        <p className="mt-2 text-gray-600">
          Configure server metadata and system settings for the MCP server.
        </p>
      </div>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2">
          <Card title="Server Settings">
            <div className="space-y-6">
              <div>
                <label htmlFor="serverName" className="block text-sm font-medium text-gray-700 mb-2">
                  Server Name
                </label>
                <input
                  type="text"
                  id="serverName"
                  value={serverName}
                  onChange={(e) => setServerName(e.target.value)}
                  className="w-full px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
                  placeholder="e.g., Odoo MCP Server"
                />
                <p className="text-xs text-gray-500 mt-1">Display name shown to MCP clients</p>
              </div>

              <div>
                <label htmlFor="instructions" className="block text-sm font-medium text-gray-700 mb-2">
                  Instructions
                </label>
                <textarea
                  id="instructions"
                  value={instructions}
                  onChange={(e) => setInstructions(e.target.value)}
                  rows={6}
                  className="w-full px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all font-mono text-sm"
                  placeholder="System instructions for the AI assistant..."
                />
                <p className="text-xs text-gray-500 mt-1">System instructions for the AI assistant</p>
              </div>

              <div>
                <label htmlFor="protocolVersion" className="block text-sm font-medium text-gray-700 mb-2">
                  Protocol Version Default
                </label>
                <input
                  type="text"
                  id="protocolVersion"
                  value={protocolVersionDefault}
                  onChange={(e) => setProtocolVersionDefault(e.target.value)}
                  className="w-full px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all font-mono"
                  placeholder="e.g., 2024-11-05"
                />
                <p className="text-xs text-gray-500 mt-1">Default MCP protocol version</p>
              </div>

              {customFields.length > 0 && (
                <div className="pt-4 border-t border-gray-200">
                  <h4 className="text-sm font-semibold text-gray-900 mb-3">Custom Fields</h4>
                  <div className="space-y-3">
                    {customFields.map((field, index) => (
                      <div key={index} className="flex gap-2">
                        <input
                          type="text"
                          value={field.key}
                          onChange={(e) => updateCustomField(index, 'key', e.target.value)}
                          className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
                          placeholder="Field name"
                        />
                        <input
                          type="text"
                          value={field.value}
                          onChange={(e) => updateCustomField(index, 'value', e.target.value)}
                          className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
                          placeholder="Value"
                        />
                        <button
                          onClick={() => removeCustomField(index)}
                          className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                          title="Remove field"
                        >
                          <Trash2 size={18} />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              <Button
                onClick={addCustomField}
                icon={<Plus size={16} />}
                variant="secondary"
                className="w-full"
              >
                Add Custom Field
              </Button>

              <div className="flex gap-3 pt-4 border-t border-gray-200">
                <Button
                  onClick={handleSave}
                  loading={loading}
                  icon={<Save size={16} />}
                  variant="primary"
                >
                  Save Configuration
                </Button>
                <Button
                  onClick={loadServer}
                  loading={loading}
                  icon={<RefreshCw size={16} />}
                  variant="secondary"
                >
                  Refresh
                </Button>
              </div>
            </div>
          </Card>
        </div>

        <div>
          <Card title="Configuration Reference">
            <div className="space-y-4">
              <div className="flex items-start gap-2">
                <Info size={16} className="text-blue-600 mt-0.5 flex-shrink-0" />
                <div>
                  <p className="text-sm font-medium text-gray-900">Standard Fields</p>
                  <p className="text-xs text-gray-600 mt-1">
                    Common server configuration options
                  </p>
                </div>
              </div>

              <div className="space-y-3 text-sm">
                <div className="p-3 bg-gray-50 rounded-lg">
                  <code className="text-blue-600 font-mono text-xs">serverName</code>
                  <p className="text-gray-600 mt-1 text-xs">
                    Display name shown to MCP clients
                  </p>
                </div>

                <div className="p-3 bg-gray-50 rounded-lg">
                  <code className="text-blue-600 font-mono text-xs">instructions</code>
                  <p className="text-gray-600 mt-1 text-xs">
                    System instructions for the AI assistant
                  </p>
                </div>

                <div className="p-3 bg-gray-50 rounded-lg">
                  <code className="text-blue-600 font-mono text-xs">protocolVersionDefault</code>
                  <p className="text-gray-600 mt-1 text-xs">
                    Default MCP protocol version (e.g., "2024-11-05")
                  </p>
                </div>
              </div>

              {serverName && (
                <div className="pt-4 border-t border-gray-200">
                  <p className="text-xs text-gray-500 mb-2">Current Server</p>
                  <p className="font-semibold text-gray-900">{serverName}</p>
                  {protocolVersionDefault && (
                    <p className="text-xs text-gray-600 mt-1">
                      Protocol: {protocolVersionDefault}
                    </p>
                  )}
                </div>
              )}
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
