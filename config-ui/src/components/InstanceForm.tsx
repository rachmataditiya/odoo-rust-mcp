import { useState, useEffect } from 'react';
import { Button } from './Button';
import { TextField } from './TextField';
import { Segmented } from './Segmented';
import { Switch } from './Switch';
import { Key, User, Database, Globe, Hash, Server, Lock } from 'lucide-react';

export interface InstanceData {
  url: string;
  db: string;
  apiKey?: string;
  username?: string;
  password?: string;
  version?: number | string;
  readOnly?: boolean;
}

export interface InstanceFormProps {
  instanceName: string | null;
  instanceData: InstanceData | null;
  existingNames: string[];
  onSave: (name: string, data: InstanceData) => void;
  onCancel: () => void;
}

type AuthType = 'apiKey' | 'userPass';

export function InstanceForm({
  instanceName,
  instanceData,
  existingNames,
  onSave,
  onCancel,
}: InstanceFormProps) {
  const [name, setName] = useState('');
  const [url, setUrl] = useState('');
  const [db, setDb] = useState('');
  const [authType, setAuthType] = useState<AuthType>('apiKey');
  const [apiKey, setApiKey] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [version, setVersion] = useState('');
  const [readOnly, setReadOnly] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (instanceName && instanceData) {
      setName(instanceName);
      setUrl(instanceData.url || '');
      setDb(instanceData.db || '');
      setVersion(instanceData.version ? String(instanceData.version) : '');
      setReadOnly(Boolean(instanceData.readOnly));

      if (instanceData.apiKey) {
        setAuthType('apiKey');
        setApiKey(instanceData.apiKey);
        setUsername('');
        setPassword('');
      } else {
        setAuthType('userPass');
        setUsername(instanceData.username || '');
        setPassword(instanceData.password || '');
        setApiKey('');
      }
    } else {
      setName('');
      setUrl('');
      setDb('');
      setAuthType('apiKey');
      setApiKey('');
      setUsername('');
      setPassword('');
      setVersion('');
      setReadOnly(false);
    }
    setErrors({});
  }, [instanceName, instanceData]);

  const validate = () => {
    const newErrors: Record<string, string> = {};

    if (!name.trim()) {
      newErrors.name = 'Instance identifier is required';
    } else if (
      existingNames.includes(name.trim()) &&
      (!instanceName || name.trim() !== instanceName)
    ) {
      newErrors.name = 'An instance with this name already exists';
    }

    if (!url.trim()) {
      newErrors.url = 'Server URL is required';
    } else {
      try {
        new URL(url);
      } catch {
        newErrors.url = 'Must be a valid URL (e.g. https://my-odoo.example.com)';
      }
    }

    if (!db.trim()) {
      newErrors.db = 'Database name is required';
    }

    if (authType === 'apiKey') {
      if (!apiKey.trim()) {
        newErrors.apiKey = 'API key is required';
      }
    } else {
      if (!username.trim()) {
        newErrors.username = 'Username is required';
      }
      if (!password.trim()) {
        newErrors.password = 'Password is required';
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;

    const data: InstanceData = {
      url: url.trim(),
      db: db.trim(),
    };

    if (authType === 'apiKey') {
      data.apiKey = apiKey.trim();
    } else {
      data.username = username.trim();
      data.password = password;
    }

    if (version.trim()) {
      const numVersion = parseInt(version.trim(), 10);
      data.version = isNaN(numVersion) ? version.trim() : numVersion;
    }

    if (readOnly) {
      data.readOnly = true;
    }

    onSave(name.trim(), data);
  };

  return (
    <form onSubmit={handleSubmit} className="p-5 sm:p-6 space-y-5">
      {/* 2-Column Grid: General Details */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-5 gap-y-4">
        {/* Instance Name */}
        <div>
          <label htmlFor="instance-name" className="block text-[12px] font-medium text-fg mb-1.5">
            Instance Name <span className="text-down">*</span>
          </label>
          <TextField
            id="instance-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. production, staging"
            mono
            icon={<Hash size={14} />}
          />
          {errors.name ? (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.name}</p>
          ) : (
            <p className="mt-1 text-[11px] text-muted">Unique internal identifier</p>
          )}
        </div>

        {/* Server URL */}
        <div>
          <label htmlFor="instance-url" className="block text-[12px] font-medium text-fg mb-1.5">
            Odoo URL <span className="text-down">*</span>
          </label>
          <TextField
            id="instance-url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://odoo.company.com"
            icon={<Globe size={14} />}
          />
          {errors.url ? (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.url}</p>
          ) : (
            <p className="mt-1 text-[11px] text-muted">Host URL of your Odoo server</p>
          )}
        </div>

        {/* Database Name */}
        <div>
          <label htmlFor="instance-db" className="block text-[12px] font-medium text-fg mb-1.5">
            Database <span className="text-down">*</span>
          </label>
          <TextField
            id="instance-db"
            value={db}
            onChange={(e) => setDb(e.target.value)}
            placeholder="e.g. odoo_prod"
            icon={<Database size={14} />}
          />
          {errors.db ? (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.db}</p>
          ) : (
            <p className="mt-1 text-[11px] text-muted">Target database name</p>
          )}
        </div>

        {/* Odoo Version */}
        <div>
          <label htmlFor="instance-version" className="block text-[12px] font-medium text-fg mb-1.5">
            Odoo Version <span className="text-[11px] text-muted font-normal">(Optional)</span>
          </label>
          <TextField
            id="instance-version"
            value={version}
            onChange={(e) => setVersion(e.target.value)}
            placeholder="e.g. 19, 18, 17"
            icon={<Server size={14} />}
          />
          <p className="mt-1 text-[11px] text-muted">Major version (e.g. 19 for JSON-2)</p>
        </div>
      </div>

      {/* Authentication Section */}
      <div className="rounded-xl border border-line bg-surface-2/40 p-4 space-y-3.5">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-line">
          <div>
            <div className="flex items-center gap-1.5 text-[12px] font-semibold text-fg">
              <Key size={13} className="text-primary" />
              <span>Authentication Credentials</span>
            </div>
            <p className="text-[11px] text-muted mt-0.5">
              Select API Key for Odoo 19+, or Username & Password for Odoo 14–18
            </p>
          </div>
          <Segmented<AuthType>
            options={[
              { label: 'API Key (19+)', value: 'apiKey' },
              { label: 'User & Password (< 19)', value: 'userPass' },
            ]}
            value={authType}
            onChange={(val) => setAuthType(val)}
            className="w-full sm:w-60"
          />
        </div>

        {authType === 'apiKey' ? (
          <div>
            <label htmlFor="instance-api-key" className="block text-[12px] font-medium text-fg mb-1.5">
              API Key Token <span className="text-down">*</span>
            </label>
            <TextField
              id="instance-api-key"
              secret
              mono
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Paste Odoo 19+ external API Key"
              icon={<Key size={14} />}
            />
            {errors.apiKey ? (
              <p className="mt-1 text-[11px] text-down font-medium">{errors.apiKey}</p>
            ) : (
              <p className="mt-1 text-[11px] text-muted">Generated from Odoo user profile preferences</p>
            )}
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label htmlFor="instance-username" className="block text-[12px] font-medium text-fg mb-1.5">
                Username / Email <span className="text-down">*</span>
              </label>
              <TextField
                id="instance-username"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="admin@example.com"
                icon={<User size={14} />}
              />
              {errors.username && (
                <p className="mt-1 text-[11px] text-down font-medium">{errors.username}</p>
              )}
            </div>

            <div>
              <label htmlFor="instance-password" className="block text-[12px] font-medium text-fg mb-1.5">
                Password / RPC Secret <span className="text-down">*</span>
              </label>
              <TextField
                id="instance-password"
                secret
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Account password"
              />
              {errors.password && (
                <p className="mt-1 text-[11px] text-down font-medium">{errors.password}</p>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Safety & Read-Only Toggle */}
      <div className="flex items-center justify-between gap-4 rounded-xl border border-line bg-surface-2/30 px-4 py-3">
        <div className="min-w-0">
          <div className="flex items-center gap-1.5">
            <Lock size={13} className={readOnly ? 'text-warning' : 'text-muted'} />
            <span className="text-[12px] font-semibold text-fg">Read-Only Safety Guard</span>
            {readOnly ? (
              <span className="text-[10px] font-medium px-1.5 py-0.2 rounded bg-warning/10 text-warning border border-warning/20">
                PROD SAFE
              </span>
            ) : (
              <span className="text-[10px] font-medium px-1.5 py-0.2 rounded bg-surface-2 text-muted border border-line">
                READ-WRITE
              </span>
            )}
          </div>
          <p className="text-[11px] text-muted mt-0.5">
            {readOnly
              ? 'All create, write, unlink, execute, and cleanup mutations are strictly blocked.'
              : 'Allows read, search, write, create, and workflow mutations on this instance.'}
          </p>
        </div>
        <div className="shrink-0 flex items-center gap-2">
          <Switch checked={readOnly} onChange={setReadOnly} />
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex items-center justify-end gap-2.5 pt-3 border-t border-line">
        <Button type="button" onClick={onCancel} variant="plain" size="md">
          Cancel
        </Button>
        <Button type="submit" variant="primary" size="md">
          {instanceName ? 'Update Instance' : 'Create Instance'}
        </Button>
      </div>
    </form>
  );
}
