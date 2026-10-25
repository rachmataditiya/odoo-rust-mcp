import { useEffect, useState } from 'react';
import { Save, RefreshCw, Plus, Trash2, ShieldCheck, Terminal, Cpu } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Card } from '../Card';
import { Button } from '../Button';
import { Field } from '../Field';
import { TextField } from '../TextField';
import { StatusMessage } from '../StatusMessage';
import { PageHeader } from '../PageHeader';
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
      const data = (await load()) as ServerConfig;
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
      if (protocolVersionDefault.trim())
        config.protocolVersionDefault = protocolVersionDefault.trim();

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
      <PageHeader
        title="Server Configuration"
        description="Manage server identity, AI prompt guidance, and protocol settings."
      >
        <Button
          onClick={loadServer}
          variant="plain"
          size="md"
          icon={<RefreshCw size={14} className={loading ? 'animate-spin' : ''} />}
          disabled={loading}
        >
          Reload
        </Button>
        <Button
          onClick={handleSave}
          variant="primary"
          size="md"
          icon={<Save size={14} />}
          disabled={loading}
          loading={loading}
        >
          Save Changes
        </Button>
      </PageHeader>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          {/* General Server Settings */}
          <Card
            title="Server Identity"
            description="Core identifiers announced to Model Context Protocol clients"
            divided
          >
            <Field
              label="Server Name"
              help="Identifier transmitted to clients during MCP protocol initialization."
              htmlFor="serverName"
            >
              <TextField
                id="serverName"
                value={serverName}
                onChange={(e) => setServerName(e.target.value)}
                placeholder="e.g. odoo-mcp-server"
                containerClassName="sm:w-72"
              />
            </Field>

            <Field
              label="Default Protocol Version"
              help="MCP wire protocol version advertised to connecting clients."
              htmlFor="protocolVersionDefault"
            >
              <TextField
                id="protocolVersionDefault"
                value={protocolVersionDefault}
                onChange={(e) => setProtocolVersionDefault(e.target.value)}
                placeholder="e.g. 2024-11-05"
                mono
                containerClassName="sm:w-72"
              />
            </Field>
          </Card>

          {/* AI System Instructions */}
          <Card
            title="System Instructions"
            description="Guidance injected into the AI model when utilizing this MCP server."
          >
            <div className="space-y-2">
              <label
                htmlFor="instructions"
                className="block text-[13px] font-medium text-fg"
              >
                MCP Prompt Instructions
              </label>
              <textarea
                id="instructions"
                rows={5}
                value={instructions}
                onChange={(e) => setInstructions(e.target.value)}
                placeholder="Enter instructions for AI models interacting with Odoo (e.g., prefer search_read with active=True, verify record state before confirming)..."
                className="w-full rounded-md border border-line bg-surface-2 p-3 text-[13px] text-fg transition-colors outline-none placeholder:text-faint hover:border-line-strong focus:border-primary focus:ring-1 focus:ring-primary font-mono leading-relaxed"
              />
              <p className="text-[12px] text-muted">
                Tip: Specific constraints on database IDs, safety checks, or common model names help Claude and Cursor query Odoo accurately.
              </p>
            </div>
          </Card>

          {/* Custom Properties */}
          <Card
            title="Custom Properties"
            description="Arbitrary additional key-value configurations stored in server.json."
            aside={
              <Button
                onClick={addCustomField}
                variant="plain"
                size="sm"
                icon={<Plus size={13} />}
              >
                Add Field
              </Button>
            }
          >
            {customFields.length === 0 ? (
              <div className="py-8 text-center text-[13px] text-muted">
                No custom fields configured. Click "Add Field" to define custom parameters.
              </div>
            ) : (
              <div className="divide-y divide-line">
                {customFields.map((field, index) => (
                  <div
                    key={index}
                    className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 py-3"
                  >
                    <TextField
                      placeholder="Property Key"
                      value={field.key}
                      onChange={(e) => updateCustomField(index, 'key', e.target.value)}
                      mono
                      containerClassName="sm:w-1/3"
                    />
                    <TextField
                      placeholder="Property Value"
                      value={field.value}
                      onChange={(e) => updateCustomField(index, 'value', e.target.value)}
                      containerClassName="flex-1"
                    />
                    <button
                      type="button"
                      onClick={() => removeCustomField(index)}
                      className="grid h-8 w-8 place-items-center rounded-md border border-line bg-surface-2 text-muted hover:text-down hover:border-down transition-colors shrink-0"
                      title="Remove field"
                      aria-label="Remove field"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </Card>
        </div>

        {/* Right Info Column */}
        <div className="space-y-6">
          <Card title="Transport Overview" description="Active communications layer">
            <div className="space-y-3.5 text-[13px]">
              <div className="flex items-start gap-2.5">
                <Terminal size={16} className="text-primary mt-0.5 shrink-0" />
                <div>
                  <div className="font-medium text-fg">Stdio Transport</div>
                  <div className="text-[12px] text-muted mt-0.5">
                    Standard input/output for local Cursor and Claude Desktop integrations.
                  </div>
                </div>
              </div>

              <div className="flex items-start gap-2.5">
                <Cpu size={16} className="text-primary mt-0.5 shrink-0" />
                <div>
                  <div className="font-medium text-fg">HTTP & Streamable SSE</div>
                  <div className="text-[12px] text-muted mt-0.5">
                    Remote daemon mode on port 3008 with Web UI and config hot-reloading.
                  </div>
                </div>
              </div>

              <div className="flex items-start gap-2.5">
                <ShieldCheck size={16} className="text-ok mt-0.5 shrink-0" />
                <div>
                  <div className="font-medium text-fg">Atomic Hot Reload</div>
                  <div className="text-[12px] text-muted mt-0.5">
                    Changes saved here sync without restarting client sessions or daemon processes.
                  </div>
                </div>
              </div>
            </div>
          </Card>

          <Card title="Peugeot 3008 Edition" description="Design & Performance">
            <div className="space-y-2 text-[12px] text-muted leading-relaxed">
              <p>
                Inspired by the sharp geometry and high-performance cockpit of the Peugeot 3008,
                this MCP server provides dual-engine Odoo connectivity:
              </p>
              <ul className="list-disc pl-4 space-y-1 text-fg/90">
                <li><strong className="text-primary">JSON-2 API:</strong> Native for Odoo 19+ with API key tokens.</li>
                <li><strong className="text-primary">JSON-RPC:</strong> Full compatibility for Odoo 14–18.</li>
                <li><strong className="text-primary">Metadata Cache:</strong> High-speed TTL cache for field structures.</li>
              </ul>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
