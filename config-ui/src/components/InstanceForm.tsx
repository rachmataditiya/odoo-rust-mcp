import React, { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import { Button } from './Button';

interface InstanceData {
  url: string;
  db: string;
  apiKey?: string;
  username?: string;
  password?: string;
  version?: string;
}

interface InstanceFormProps {
  instanceName: string | null;
  instanceData: InstanceData | null;
  existingNames: string[];
  onSave: (name: string, data: InstanceData) => void;
  onCancel: () => void;
}

type AuthType = 'apiKey' | 'userPass';

export function InstanceForm({ instanceName, instanceData, existingNames, onSave, onCancel }: InstanceFormProps) {
  const [name, setName] = useState('');
  const [url, setUrl] = useState('');
  const [db, setDb] = useState('');
  const [authType, setAuthType] = useState<AuthType>('userPass');
  const [apiKey, setApiKey] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [version, setVersion] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (instanceName && instanceData) {
      setName(instanceName);
      setUrl(instanceData.url);
      setDb(instanceData.db);
      setVersion(instanceData.version || '');

      if (instanceData.apiKey) {
        setAuthType('apiKey');
        setApiKey(instanceData.apiKey);
      } else {
        setAuthType('userPass');
        setUsername(instanceData.username || '');
        setPassword(instanceData.password || '');
      }
    } else {
      setName('');
      setUrl('');
      setDb('');
      setAuthType('userPass');
      setApiKey('');
      setUsername('');
      setPassword('');
      setVersion('');
    }
    setErrors({});
  }, [instanceName, instanceData]);

  const validate = () => {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Instance name is required';
    } else if (!instanceName && existingNames.includes(name.trim())) {
      newErrors.name = 'Instance name already exists';
    }

    if (!url.trim()) {
      newErrors.url = 'URL is required';
    }

    if (!db.trim()) {
      newErrors.db = 'Database name is required';
    }

    if (authType === 'apiKey' && !apiKey.trim()) {
      newErrors.apiKey = 'API Key is required when using API Key authentication';
    }

    if (authType === 'userPass') {
      if (!username.trim()) {
        newErrors.username = 'Username is required when using username/password authentication';
      }
      if (!password.trim()) {
        newErrors.password = 'Password is required when using username/password authentication';
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) {
      return;
    }

    const data: InstanceData = {
      url: url.trim(),
      db: db.trim(),
    };

    if (authType === 'apiKey') {
      data.apiKey = apiKey.trim();
    } else {
      data.username = username.trim();
      data.password = password.trim();
    }

    if (version.trim()) {
      data.version = version.trim();
    }

    onSave(name.trim(), data);
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-xl max-w-2xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <h3 className="text-xl font-semibold text-gray-900">
            {instanceName ? 'Edit Instance' : 'Add New Instance'}
          </h3>
          <button
            onClick={onCancel}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X size={24} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto">
          <div className="p-6 space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Instance Name <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                disabled={!!instanceName}
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  errors.name ? 'border-red-500' : 'border-gray-300'
                } ${instanceName ? 'bg-gray-100 cursor-not-allowed' : ''}`}
                placeholder="e.g., production, local, staging"
              />
              {errors.name && <p className="mt-1 text-sm text-red-600">{errors.name}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                URL <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  errors.url ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="e.g., https://odoo.example.com"
              />
              {errors.url && <p className="mt-1 text-sm text-red-600">{errors.url}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Database Name <span className="text-red-500">*</span>
              </label>
              <input
                type="text"
                value={db}
                onChange={(e) => setDb(e.target.value)}
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  errors.db ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="e.g., production_db"
              />
              {errors.db && <p className="mt-1 text-sm text-red-600">{errors.db}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Authentication Method <span className="text-red-500">*</span>
              </label>
              <div className="flex gap-4">
                <label className="flex items-center">
                  <input
                    type="radio"
                    value="userPass"
                    checked={authType === 'userPass'}
                    onChange={(e) => setAuthType(e.target.value as AuthType)}
                    className="mr-2"
                  />
                  <span className="text-sm text-gray-700">Username & Password</span>
                </label>
                <label className="flex items-center">
                  <input
                    type="radio"
                    value="apiKey"
                    checked={authType === 'apiKey'}
                    onChange={(e) => setAuthType(e.target.value as AuthType)}
                    className="mr-2"
                  />
                  <span className="text-sm text-gray-700">API Key</span>
                </label>
              </div>
            </div>

            {authType === 'userPass' ? (
              <>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Username <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="text"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                      errors.username ? 'border-red-500' : 'border-gray-300'
                    }`}
                    placeholder="e.g., admin@example.com"
                  />
                  {errors.username && <p className="mt-1 text-sm text-red-600">{errors.username}</p>}
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Password <span className="text-red-500">*</span>
                  </label>
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                      errors.password ? 'border-red-500' : 'border-gray-300'
                    }`}
                    placeholder="Enter password"
                  />
                  {errors.password && <p className="mt-1 text-sm text-red-600">{errors.password}</p>}
                </div>
              </>
            ) : (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  API Key <span className="text-red-500">*</span>
                </label>
                <input
                  type="text"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm ${
                    errors.apiKey ? 'border-red-500' : 'border-gray-300'
                  }`}
                  placeholder="Enter API key"
                />
                {errors.apiKey && <p className="mt-1 text-sm text-red-600">{errors.apiKey}</p>}
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Odoo Version
              </label>
              <input
                type="text"
                value={version}
                onChange={(e) => setVersion(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="e.g., 16, 17, 18"
              />
            </div>
          </div>

          <div className="px-6 py-4 border-t border-gray-200 flex justify-end gap-3 bg-gray-50">
            <Button type="button" variant="ghost" onClick={onCancel}>
              Cancel
            </Button>
            <Button type="submit" variant="primary">
              {instanceName ? 'Update Instance' : 'Add Instance'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
