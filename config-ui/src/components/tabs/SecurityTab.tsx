import React, { useState, useEffect } from 'react';
import {
  Shield,
  Lock,
  Copy,
  Check,
  AlertTriangle,
  RefreshCw,
  CheckCircle2,
  Info,
} from 'lucide-react';
import { useAuth } from '../AuthProvider';
import { useSecurityActions } from '../../hooks/useAuth';
import { Card } from '../Card';
import { Field } from '../Field';
import { TextField } from '../TextField';
import { Switch } from '../Switch';
import { Button } from '../Button';
import { PageHeader } from '../PageHeader';

export function SecurityTab() {
  const { username, authEnabled } = useAuth();
  const {
    changePassword,
    getMcpAuthStatus,
    setMcpAuthEnabled,
    generateMcpToken,
    loading,
  } = useSecurityActions();

  // Password change state
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [passwordSuccess, setPasswordSuccess] = useState(false);

  // MCP Auth state
  const [mcpAuthEnabled, setMcpAuthEnabledState] = useState(false);
  const [mcpTokenConfigured, setMcpTokenConfigured] = useState(false);
  const [generatedToken, setGeneratedToken] = useState<string | null>(null);
  const [tokenCopied, setTokenCopied] = useState(false);
  const [mcpError, setMcpError] = useState<string | null>(null);
  const [mcpLoading, setMcpLoading] = useState(false);
  const [toggleLoading, setToggleLoading] = useState(false);

  // Load MCP auth status on mount
  useEffect(() => {
    getMcpAuthStatus()
      .then((status) => {
        setMcpAuthEnabledState(status.enabled);
        setMcpTokenConfigured(status.token_configured);
      })
      .catch(console.error);
  }, [getMcpAuthStatus]);

  const handlePasswordChange = async (e: React.FormEvent) => {
    e.preventDefault();
    setPasswordError(null);
    setPasswordSuccess(false);

    if (newPassword.length < 4) {
      setPasswordError('Password must be at least 4 characters');
      return;
    }

    if (newPassword !== confirmPassword) {
      setPasswordError('Passwords do not match');
      return;
    }

    try {
      await changePassword(newPassword);
      setPasswordSuccess(true);
      setNewPassword('');
      setConfirmPassword('');
      setTimeout(() => setPasswordSuccess(false), 3000);
    } catch (err) {
      setPasswordError(
        err instanceof Error ? err.message : 'Failed to change password',
      );
    }
  };

  const handleToggleMcpAuth = async (newValue: boolean) => {
    setToggleLoading(true);
    setMcpError(null);

    try {
      await setMcpAuthEnabled(newValue);
      setMcpAuthEnabledState(newValue);
    } catch (err) {
      setMcpError(
        err instanceof Error ? err.message : 'Failed to update setting',
      );
    } finally {
      setToggleLoading(false);
    }
  };

  const handleGenerateToken = async () => {
    setMcpError(null);
    setGeneratedToken(null);
    setMcpLoading(true);

    try {
      const token = await generateMcpToken();
      setGeneratedToken(token);
      setMcpTokenConfigured(true);
    } catch (err) {
      setMcpError(
        err instanceof Error ? err.message : 'Failed to generate token',
      );
    } finally {
      setMcpLoading(false);
    }
  };

  const copyToken = async () => {
    if (generatedToken) {
      await navigator.clipboard.writeText(generatedToken);
      setTokenCopied(true);
      setTimeout(() => setTokenCopied(false), 2000);
    }
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="Security & Access"
        description="Configure admin dashboard credentials and MCP bearer token enforcement."
      />

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 space-y-6">
          {/* MCP Client Authentication */}
          <Card
            title="MCP Bearer Authentication"
            description="Require Authorization: Bearer <token> on all MCP endpoints (/mcp, /sse, /ws)."
            divided
          >
            <Field
              label="Enforce MCP Token"
              help="When enabled, any connecting AI client must supply a valid authentication token."
            >
              <Switch
                checked={mcpAuthEnabled}
                onChange={handleToggleMcpAuth}
                busy={toggleLoading}
                label="Toggle MCP Authentication"
              />
            </Field>

            <Field
              label="Token Status"
              help="Whether a valid shared secret token has been provisioned."
            >
              <span
                className={`inline-flex items-center gap-1.5 text-[12px] font-medium px-2.5 py-1 rounded-md border ${
                  mcpTokenConfigured
                    ? 'bg-ok/10 text-ok border-ok/25'
                    : 'bg-warn/10 text-warn border-warn/25'
                }`}
              >
                {mcpTokenConfigured ? (
                  <>
                    <CheckCircle2 size={13} /> Token Configured
                  </>
                ) : (
                  <>
                    <AlertTriangle size={13} /> Token Missing
                  </>
                )}
              </span>
            </Field>

            {/* Token Generation / Viewing */}
            <div className="px-5 py-4 space-y-3">
              <div className="flex items-center justify-between gap-3">
                <span className="text-[13px] font-medium text-fg">
                  Security Token
                </span>
                <Button
                  onClick={handleGenerateToken}
                  variant="plain"
                  size="sm"
                  loading={mcpLoading}
                  icon={<RefreshCw size={13} />}
                >
                  Generate New Token
                </Button>
              </div>

              {mcpError && (
                <div className="p-3 bg-down/10 border border-down/20 rounded-xl text-down text-[12px]">
                  {mcpError}
                </div>
              )}

              {generatedToken ? (
                <div className="space-y-2">
                  <div className="flex items-center gap-2 p-2.5 rounded-lg border border-line bg-surface-2">
                    <code className="font-mono text-[12px] text-primary flex-1 break-all select-all">
                      {generatedToken}
                    </code>
                    <button
                      type="button"
                      onClick={copyToken}
                      className="grid h-8 w-8 place-items-center rounded-md border border-line bg-surface text-muted hover:text-fg transition-colors shrink-0"
                      title={tokenCopied ? 'Copied' : 'Copy token'}
                      aria-label="Copy token"
                    >
                      {tokenCopied ? (
                        <Check size={14} className="text-ok" />
                      ) : (
                        <Copy size={14} />
                      )}
                    </button>
                  </div>
                  <p className="text-[11px] text-warn">
                    Make sure to copy this token now. It will not be displayed again once you navigate away.
                  </p>
                </div>
              ) : (
                <p className="text-[12px] text-muted">
                  Generate a cryptographically secure random bearer token to authenticate remote Claude / Cursor connections.
                </p>
              )}
            </div>
          </Card>

          {/* Admin Dashboard Password */}
          {authEnabled && (
            <Card
              title="Change Admin Password"
              description={`Update login credentials for user "${username || 'admin'}"`}
            >
              <form onSubmit={handlePasswordChange} className="space-y-4">
                {passwordSuccess && (
                  <div className="flex items-center gap-2 p-3 bg-ok/10 border border-ok/25 rounded-xl text-ok text-[13px]">
                    <CheckCircle2 size={15} />
                    <span>Password updated successfully!</span>
                  </div>
                )}

                {passwordError && (
                  <div className="flex items-center gap-2 p-3 bg-down/10 border border-down/25 rounded-xl text-down text-[13px]">
                    <AlertTriangle size={15} />
                    <span>{passwordError}</span>
                  </div>
                )}

                <div>
                  <label
                    htmlFor="new-password"
                    className="block text-[13px] font-medium text-fg mb-1.5"
                  >
                    New Password
                  </label>
                  <TextField
                    id="new-password"
                    type="password"
                    secret
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    placeholder="Enter new password (min. 4 characters)"
                    disabled={loading}
                  />
                </div>

                <div>
                  <label
                    htmlFor="confirm-password"
                    className="block text-[13px] font-medium text-fg mb-1.5"
                  >
                    Confirm Password
                  </label>
                  <TextField
                    id="confirm-password"
                    type="password"
                    secret
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder="Re-enter new password"
                    disabled={loading}
                  />
                </div>

                <div className="flex justify-end pt-2">
                  <Button
                    type="submit"
                    variant="primary"
                    size="md"
                    loading={loading}
                    disabled={!newPassword || !confirmPassword || loading}
                    icon={<Lock size={14} />}
                  >
                    Update Password
                  </Button>
                </div>
              </form>
            </Card>
          )}
        </div>

        {/* Right Info Column */}
        <div className="space-y-6">
          <Card title="Security Hardening" description="Recommended best practices">
            <div className="space-y-3 text-[12px] text-muted leading-relaxed">
              <div className="flex items-start gap-2.5">
                <Shield size={16} className="text-primary mt-0.5 shrink-0" />
                <div>
                  <div className="font-semibold text-fg">Dedicated Bot User</div>
                  <div className="mt-0.5">
                    Never use administrative Odoo superuser accounts. Create a dedicated user with minimal read/write access.
                  </div>
                </div>
              </div>

              <div className="flex items-start gap-2.5">
                <Lock size={16} className="text-primary mt-0.5 shrink-0" />
                <div>
                  <div className="font-semibold text-fg">API Keys over Passwords</div>
                  <div className="mt-0.5">
                    For Odoo 19+, always use generated API keys rather than raw passwords. API keys can be revoked immediately.
                  </div>
                </div>
              </div>

              <div className="flex items-start gap-2.5">
                <Info size={16} className="text-primary mt-0.5 shrink-0" />
                <div>
                  <div className="font-semibold text-fg">Write Tool Guards</div>
                  <div className="mt-0.5">
                    Mutating operations (create, write, unlink) are guarded behind environment flags for protection against hallucinations.
                  </div>
                </div>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
