import { useState, type FormEvent } from 'react';
import { Lock, User, AlertCircle } from 'lucide-react';
import { useAuth } from './AuthProvider';
import { Brand } from './Brand';
import { TextField } from './TextField';
import { Button } from './Button';

export function LoginForm() {
  const { login } = useAuth();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      await login(username, password);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-base flex flex-col items-center justify-center p-4">
      <div className="w-full max-w-sm">
        <div className="overflow-hidden rounded-2xl border border-line bg-surface shadow-xl p-7">
          <div className="flex flex-col items-center text-center mb-6">
            <div className="mb-3">
              <Brand />
            </div>
            <p className="text-[13px] text-muted">
              Sign in to manage MCP server settings
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="flex items-center gap-2 p-3 bg-down/10 border border-down/20 rounded-xl text-down text-[13px]">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>{error}</span>
              </div>
            )}

            <div>
              <label
                htmlFor="username"
                className="block text-[13px] font-medium text-fg mb-1.5"
              >
                Username
              </label>
              <TextField
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="Enter username"
                icon={<User size={15} />}
                required
                disabled={loading}
              />
            </div>

            <div>
              <label
                htmlFor="password"
                className="block text-[13px] font-medium text-fg mb-1.5"
              >
                Password
              </label>
              <TextField
                id="password"
                type="password"
                secret
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter password"
                icon={<Lock size={15} />}
                required
                disabled={loading}
              />
            </div>

            <div className="pt-2">
              <Button
                type="submit"
                disabled={loading}
                loading={loading}
                variant="primary"
                size="md"
                className="w-full py-2.5"
              >
                Sign In
              </Button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
