import { useState, useEffect } from 'react';
import { Button } from './Button';
import { Field } from './Field';
import { TextField } from './TextField';
import { Segmented } from './Segmented';
import { Switch } from './Switch';
import { Key, User, Database, Globe, Hash, Lock } from 'lucide-react';

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
        newErrors.url = 'Must be a valid URL (e.g., https://my-odoo.example.com)';
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
    <form onSubmit={handleSubmit} className="divide-y divide-line">
      {/* Instance Name */}
      <Field
        label="Instance Name"
        help="Unique internal identifier (e.g. production, staging, demo)."
        htmlFor="instance-name"
      >
        <div className="w-full sm:w-64">
          <TextField
            id="instance-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. production"
            mono
            icon={<Hash size={14} />}
          />
          {errors.name && (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.name}</p>
          )}
        </div>
      </Field>

      {/* Server URL */}
      <Field
        label="Odoo URL"
        help="Base host URL where your Odoo instance is hosted."
        htmlFor="instance-url"
      >
        <div className="w-full sm:w-64">
          <TextField
            id="instance-url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://odoo.company.com"
            icon={<Globe size={14} />}
          />
          {errors.url && (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.url}</p>
          )}
        </div>
      </Field>

      {/* Database Name */}
      <Field
        label="Database"
        help="Odoo database name corresponding to this instance."
        htmlFor="instance-db"
      >
        <div className="w-full sm:w-64">
          <TextField
            id="instance-db"
            value={db}
            onChange={(e) => setDb(e.target.value)}
            placeholder="e.g. odoo_prod"
            icon={<Database size={14} />}
          />
          {errors.db && (
            <p className="mt-1 text-[11px] text-down font-medium">{errors.db}</p>
          )}
        </div>
      </Field>

      {/* Authentication Mode */}
      <Field
        label="Authentication Mode"
        help="Select API Key for modern Odoo 19+, or Username/Password for Odoo 14–18."
      >
        <Segmented<AuthType>
          options={[
            { label: 'API Key (Odoo 19+)', value: 'apiKey' },
            { label: 'User & Password (< 19)', value: 'userPass' },
          ]}
          value={authType}
          onChange={(val) => setAuthType(val)}
          className="w-full sm:w-64"
        />
      </Field>

      {/* Auth Credentials */}
      {authType === 'apiKey' ? (
        <Field
          label="API Key"
          help="Odoo 19+ external API Key generated from user profile."
          htmlFor="instance-api-key"
        >
          <div className="w-full sm:w-64">
            <TextField
              id="instance-api-key"
              secret
              mono
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="API key token"
              icon={<Key size={14} />}
            />
            {errors.apiKey && (
              <p className="mt-1 text-[11px] text-down font-medium">{errors.apiKey}</p>
            )}
          </div>
        </Field>
      ) : (
        <>
          <Field
            label="Username"
            help="Odoo user login email or username."
            htmlFor="instance-username"
          >
            <div className="w-full sm:w-64">
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
          </Field>

          <Field
            label="Password"
            help="Odoo user account password or RPC secret."
            htmlFor="instance-password"
          >
            <div className="w-full sm:w-64">
              <TextField
                id="instance-password"
                secret
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="User password"
              />
              {errors.password && (
                <p className="mt-1 text-[11px] text-down font-medium">{errors.password}</p>
              )}
            </div>
          </Field>
        </>
      )}

      {/* Odoo Version */}
      <Field
        label="Odoo Version (Optional)"
        help="Major version number (e.g. 19, 18, 17) for API optimization."
        htmlFor="instance-version"
      >
        <TextField
          id="instance-version"
          value={version}
          onChange={(e) => setVersion(e.target.value)}
          placeholder="e.g. 19"
          containerClassName="sm:w-64"
        />
      </Field>

      {/* Read-Only Safety Mode */}
      <Field
        label="Read-Only Mode"
        help="Prohibit any write, create, unlink, workflow, or cleanup operations on this instance."
      >
        <div className="flex items-center gap-3">
          <Switch
            checked={readOnly}
            onChange={setReadOnly}
          />
          <div className="flex items-center gap-1.5 text-xs text-muted">
            <Lock size={12} className={readOnly ? 'text-warning' : 'opacity-40'} />
            <span className={readOnly ? 'font-medium text-warning' : 'font-mono'}>
              {readOnly ? 'Strictly Read-Only (Safe for Production)' : 'Read-Write allowed'}
            </span>
          </div>
        </div>
      </Field>

      {/* Form Action Buttons */}
      <div className="flex items-center justify-end gap-2.5 pt-4">
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
