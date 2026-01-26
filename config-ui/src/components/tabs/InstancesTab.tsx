import React, { useEffect, useState } from 'react';
import { Plus, Edit2, Trash2, RefreshCw, Database, Key, User } from 'lucide-react';
import { useConfig } from '../../hooks/useConfig';
import { Card } from '../Card';
import { Button } from '../Button';
import { StatusMessage } from '../StatusMessage';
import { InstanceForm } from '../InstanceForm';
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
      const data = await load() as InstanceConfig;
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

  const handleSaveInstance = async (name: string, data: any) => {
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
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900">Odoo Instances</h2>
          <p className="mt-2 text-gray-600">
            Configure your Odoo instance connections. Changes are applied immediately with hot reload.
          </p>
        </div>
        <Button
          onClick={handleAdd}
          icon={<Plus size={18} />}
          variant="primary"
          disabled={loading}
        >
          Add Instance
        </Button>
      </div>

      {status && (
        <StatusMessage
          status={status}
          onDismiss={status.type === 'error' ? () => {} : undefined}
        />
      )}

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">
            Configured Instances ({instances.length})
          </h3>
          <Button
            onClick={loadInstances}
            loading={loading}
            icon={<RefreshCw size={16} />}
            variant="ghost"
            size="sm"
          >
            Refresh
          </Button>
        </div>

        {instances.length === 0 ? (
          <div className="text-center py-16">
            <Database className="mx-auto text-gray-400 mb-3" size={48} />
            <p className="text-gray-500 mb-4">No instances configured</p>
            <Button onClick={handleAdd} icon={<Plus size={16} />} variant="primary">
              Add Your First Instance
            </Button>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-200 bg-gray-50">
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Name
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    URL
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Database
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Auth Type
                  </th>
                  <th className="px-4 py-3 text-left text-sm font-semibold text-gray-700">
                    Version
                  </th>
                  <th className="px-4 py-3 text-right text-sm font-semibold text-gray-700">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200">
                {instances.map(([name, instance]) => {
                  const authType = instance.apiKey ? 'API Key' : 'Username/Password';
                  const authIcon = instance.apiKey ? Key : User;
                  const AuthIcon = authIcon;

                  return (
                    <tr key={name} className="hover:bg-gray-50 transition-colors">
                      <td className="px-4 py-4">
                        <div className="flex items-center gap-2">
                          <Database size={16} className="text-blue-600 flex-shrink-0" />
                          <code className="font-mono text-sm font-medium text-gray-900">
                            {name}
                          </code>
                        </div>
                      </td>
                      <td className="px-4 py-4">
                        <a
                          href={instance.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-sm text-blue-600 hover:text-blue-800 hover:underline max-w-xs truncate block"
                          title={instance.url}
                        >
                          {instance.url}
                        </a>
                      </td>
                      <td className="px-4 py-4">
                        <code className="text-sm text-gray-700 font-mono">
                          {instance.db}
                        </code>
                      </td>
                      <td className="px-4 py-4">
                        <div className="flex items-center gap-1.5">
                          <AuthIcon size={14} className="text-gray-500" />
                          <span className="text-xs text-gray-600">{authType}</span>
                        </div>
                      </td>
                      <td className="px-4 py-4">
                        {instance.version ? (
                          <span className="inline-flex items-center px-2 py-1 rounded-md bg-blue-100 text-blue-700 text-xs font-medium">
                            v{instance.version}
                          </span>
                        ) : (
                          <span className="text-xs text-gray-400">-</span>
                        )}
                      </td>
                      <td className="px-4 py-4">
                        <div className="flex items-center justify-end gap-2">
                          <button
                            onClick={() => handleEdit(name)}
                            className="p-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                            title="Edit"
                          >
                            <Edit2 size={16} />
                          </button>
                          <button
                            onClick={() => handleDelete(name)}
                            className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                            title="Delete"
                          >
                            <Trash2 size={16} />
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {showForm && (
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
      )}
    </div>
  );
}
