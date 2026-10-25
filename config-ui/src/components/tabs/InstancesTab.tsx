import { useEffect, useState } from 'react';
import { Plus, Edit2, Trash2, Database, Key, User, ExternalLink, Globe, Server, Lock } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Button } from '../Button';
import { StatusMessage } from '../StatusMessage';
import { InstanceForm, type InstanceData } from '../InstanceForm';
import { Modal } from '../Modal';
import { PageHeader } from '../PageHeader';
import type { InstanceConfig } from '../../types';

export function InstancesTab() {
  const { load, save, status, loading } = useConfig('instances');
  const [config, setConfig] = useState<InstanceConfig>({});
  const [showForm, setShowForm] = useState(false);
  const [editingName, setEditingName] = useState<string | null>(null);

  useEffect(() => {
    loadInstances();
  }, []);

  const loadInstances = async () => {
    try {
      const data = (await load()) as InstanceConfig;
      setConfig(data);
    } catch (error) {
      console.error('Failed to load instances:', error);
    }
  };

  const handleAdd = () => {
    setEditingName(null);
    setShowForm(true);
  };

  const handleEdit = (name: string) => {
    setEditingName(name);
    setShowForm(true);
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`Are you sure you want to delete the instance "${name}"?`)) {
      return;
    }

    try {
      const updatedConfig = { ...config };
      delete updatedConfig[name];
      await save(updatedConfig);
      await loadInstances();
    } catch (error) {
      console.error('Failed to delete instance:', error);
    }
  };

  const handleSaveInstance = async (name: string, data: InstanceData) => {
    try {
      const updatedConfig = { ...config };

      if (editingName && editingName !== name) {
        delete updatedConfig[editingName];
      }

      updatedConfig[name] = data;

      await save(updatedConfig);
      await loadInstances();
      setShowForm(false);
      setEditingName(null);
    } catch (error) {
      console.error('Failed to save instance:', error);
    }
  };

  const instances = Object.entries(config);
  const existingNames = Object.keys(config);

  return (
    <div className="space-y-6">
      <PageHeader
        title="Odoo Instances"
        description="Configure target Odoo environments. Changes are hot-reloaded automatically into active MCP clients."
      >
        <Button
          onClick={handleAdd}
          icon={<Plus size={15} />}
          variant="primary"
          size="md"
          disabled={loading}
        >
          Add Instance
        </Button>
      </PageHeader>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      {instances.length === 0 ? (
        <div className="rounded-xl border border-dashed border-line bg-surface/50 p-12 text-center">
          <div className="mx-auto w-12 h-12 rounded-xl bg-surface-2 border border-line flex items-center justify-center text-muted mb-4 shadow-xs">
            <Database size={22} />
          </div>
          <h3 className="text-[14px] font-semibold text-fg">No Instances Configured</h3>
          <p className="mt-1 text-[13px] text-muted max-w-sm mx-auto">
            Add an Odoo connection to enable querying models, reading records, and executing workflows.
          </p>
          <div className="mt-5">
            <Button onClick={handleAdd} icon={<Plus size={15} />} variant="primary" size="md">
              Add First Instance
            </Button>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {instances.map(([name, instance]) => (
            <div
              key={name}
              className="overflow-hidden rounded-xl border border-line bg-surface shadow-sm hover:border-line-strong transition-all flex flex-col"
            >
              {/* Card Header */}
              <div className="flex items-center justify-between gap-3 border-b border-line bg-surface-2/40 px-4 py-3">
                <div className="flex items-center gap-2.5 min-w-0">
                  <div className="w-7 h-7 rounded-lg bg-primary/10 text-primary flex items-center justify-center shrink-0">
                    <Database size={15} />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-mono text-[13px] font-semibold text-fg truncate">
                        {name}
                      </span>
                      {instance.version && (
                        <span className="text-[10px] font-medium px-1.5 py-0.5 rounded bg-surface-2 text-muted border border-line">
                          v{instance.version}
                        </span>
                      )}
                      {instance.readOnly && (
                        <span
                          className="inline-flex items-center gap-1 text-[10px] font-medium px-1.5 py-0.5 rounded bg-warning/10 text-warning border border-warning/20"
                          title="Read-only instance: mutating operations prohibited"
                        >
                          <Lock size={10} />
                          ReadOnly
                        </span>
                      )}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-1 shrink-0">
                  <button
                    type="button"
                    onClick={() => handleEdit(name)}
                    className="grid h-7 w-7 place-items-center rounded-md text-muted hover:text-fg hover:bg-surface-3 transition-colors"
                    title="Edit instance"
                    aria-label="Edit instance"
                  >
                    <Edit2 size={13} />
                  </button>
                  <button
                    type="button"
                    onClick={() => handleDelete(name)}
                    className="grid h-7 w-7 place-items-center rounded-md text-muted hover:text-down hover:bg-down/10 transition-colors"
                    title="Delete instance"
                    aria-label="Delete instance"
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              </div>

              {/* Card Body */}
              <div className="p-4 space-y-3 flex-1 flex flex-col justify-between text-[13px]">
                <div className="space-y-2.5">
                  {/* Host URL */}
                  <div className="flex items-center justify-between gap-2 text-muted">
                    <div className="flex items-center gap-1.5 text-faint">
                      <Globe size={13} />
                      <span className="text-[12px]">URL</span>
                    </div>
                    <a
                      href={instance.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="font-mono text-[12px] text-fg hover:text-primary transition-colors flex items-center gap-1 truncate max-w-[65%]"
                      title={instance.url}
                    >
                      <span className="truncate">{instance.url}</span>
                      <ExternalLink size={10} className="shrink-0 opacity-60" />
                    </a>
                  </div>

                  {/* Database Name */}
                  <div className="flex items-center justify-between gap-2 text-muted">
                    <div className="flex items-center gap-1.5 text-faint">
                      <Server size={13} />
                      <span className="text-[12px]">Database</span>
                    </div>
                    <span className="font-mono text-[12px] font-medium px-2 py-0.5 rounded-md bg-surface-2 text-fg border border-line truncate max-w-[65%]">
                      {instance.db}
                    </span>
                  </div>

                  {/* Auth Mode */}
                  <div className="flex items-center justify-between gap-2 text-muted">
                    <div className="flex items-center gap-1.5 text-faint">
                      {instance.apiKey ? <Key size={13} /> : <User size={13} />}
                      <span className="text-[12px]">Auth Mode</span>
                    </div>
                    {instance.apiKey ? (
                      <span className="inline-flex items-center gap-1 text-[11px] font-medium px-2 py-0.5 rounded-md bg-primary/10 text-primary border border-primary/20">
                        <Key size={10} /> API Key (JSON-2)
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 text-[11px] font-medium px-2 py-0.5 rounded-md bg-surface-2 text-muted border border-line">
                        <User size={10} /> {instance.username || 'User/Pass (RPC)'}
                      </span>
                    )}
                  </div>
                </div>

                {/* Resource Scheme URI hint */}
                <div className="pt-2 border-t border-line/60 flex items-center justify-between text-[11px] text-muted">
                  <span>MCP Resource URI</span>
                  <code className="text-primary font-mono text-[11px]">
                    odoo://{name}/models
                  </code>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Add / Edit Instance Modal */}
      <Modal
        isOpen={showForm}
        onClose={() => {
          setShowForm(false);
          setEditingName(null);
        }}
        title={editingName ? `Edit "${editingName}"` : 'New Odoo Instance'}
        description="Enter connection and authentication credentials for this instance."
      >
        <InstanceForm
          instanceName={editingName}
          instanceData={editingName ? config[editingName] : null}
          existingNames={existingNames}
          onSave={handleSaveInstance}
          onCancel={() => {
            setShowForm(false);
            setEditingName(null);
          }}
        />
      </Modal>
    </div>
  );
}
