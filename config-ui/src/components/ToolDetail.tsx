import { useState } from 'react';
import { ChevronDown, ChevronRight, Shield, ShieldAlert, Code2 } from 'lucide-react';
import { Switch } from './Switch';
import type { ToolConfig } from '../types';

export interface ToolDetailProps {
  tool: ToolConfig;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onToggleGuard?: (guardKey: string, enabled: boolean) => void;
  availableGuards?: Array<{ key: string; value: string; label: string }>;
}

export function ToolDetail({
  tool,
  enabled,
  onToggle,
  onToggleGuard: _onToggleGuard,
  availableGuards: _availableGuards = [],
}: ToolDetailProps) {
  const [expanded, setExpanded] = useState(false);

  const inputSchema = tool.inputSchema || {};
  const properties = inputSchema.properties || {};
  const required = inputSchema.required || [];
  const hasGuards = tool.guards && Object.keys(tool.guards).length > 0;
  const propertyEntries = Object.entries(properties);

  return (
    <div
      className={`overflow-hidden rounded-xl border transition-all ${
        enabled
          ? 'border-line bg-surface shadow-xs'
          : 'border-line/60 bg-surface/40 opacity-75'
      }`}
    >
      {/* Tool Header Row */}
      <div className="flex items-center justify-between gap-4 px-4 py-3.5">
        <div className="flex items-center gap-3 min-w-0 flex-1">
          <button
            type="button"
            onClick={() => setExpanded(!expanded)}
            className="grid h-6 w-6 place-items-center rounded text-muted hover:text-fg hover:bg-surface-2 transition-colors shrink-0"
            aria-label={expanded ? 'Collapse tool details' : 'Expand tool details'}
          >
            {expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
          </button>

          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2 mb-0.5">
              <span className="font-mono text-[13px] font-semibold text-fg tracking-tight">
                {tool.name}
              </span>

              {hasGuards && (
                <span className="inline-flex items-center gap-1 text-[11px] font-medium px-2 py-0.5 rounded-md bg-warn/10 text-warn border border-warn/25">
                  <Shield size={11} /> Guarded
                </span>
              )}

              {tool.operation?.type && (
                <span className="text-[11px] font-mono px-1.5 py-0.5 rounded bg-surface-2 text-muted border border-line">
                  {tool.operation.type}
                </span>
              )}
            </div>

            <p className="text-[12px] text-muted line-clamp-1">
              {tool.description || 'No description provided.'}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3 shrink-0">
          <span
            className={`text-[11px] font-medium ${
              enabled ? 'text-ok' : 'text-faint'
            }`}
          >
            {enabled ? 'Active' : 'Disabled'}
          </span>
          <Switch checked={enabled} onChange={onToggle} label={`Toggle ${tool.name}`} />
        </div>
      </div>

      {/* Expanded Parameters & Guard Inspector */}
      {expanded && (
        <div className="border-t border-line bg-surface-2/30 px-5 py-4 space-y-4 animate-in slide-in-from-top-1 duration-150">
          {tool.description && (
            <div>
              <div className="text-[11px] font-semibold text-faint uppercase tracking-wider mb-1">
                Description
              </div>
              <p className="text-[13px] text-fg leading-relaxed">
                {tool.description}
              </p>
            </div>
          )}

          {/* Parameters Table */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="text-[11px] font-semibold text-faint uppercase tracking-wider flex items-center gap-1.5">
                <Code2 size={12} /> Input Schema Parameters ({propertyEntries.length})
              </div>
            </div>

            {propertyEntries.length === 0 ? (
              <p className="text-[12px] text-muted italic">This tool takes no parameters.</p>
            ) : (
              <div className="overflow-x-auto rounded-lg border border-line bg-surface">
                <table className="w-full text-left text-[12px]">
                  <thead>
                    <tr className="border-b border-line bg-surface-2/60 text-muted font-medium">
                      <th className="px-3 py-2">Parameter</th>
                      <th className="px-3 py-2">Type</th>
                      <th className="px-3 py-2">Required</th>
                      <th className="px-3 py-2">Description</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-line">
                    {propertyEntries.map(([propName, propDef]: [string, any]) => {
                      const isRequired = required.includes(propName);
                      return (
                        <tr key={propName} className="hover:bg-surface-2/30">
                          <td className="px-3 py-2 font-mono font-medium text-fg">
                            {propName}
                          </td>
                          <td className="px-3 py-2 text-primary font-mono text-[11px]">
                            {propDef.type || 'any'}
                          </td>
                          <td className="px-3 py-2">
                            {isRequired ? (
                              <span className="text-[10px] font-semibold text-warn bg-warn/10 px-1.5 py-0.5 rounded">
                                required
                              </span>
                            ) : (
                              <span className="text-[10px] text-muted">optional</span>
                            )}
                          </td>
                          <td className="px-3 py-2 text-muted leading-snug">
                            {propDef.description || '—'}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          {/* Security Guards Section */}
          {hasGuards && (
            <div>
              <div className="text-[11px] font-semibold text-faint uppercase tracking-wider mb-2 flex items-center gap-1.5">
                <ShieldAlert size={12} /> Safety Guards
              </div>
              <div className="p-3 rounded-lg border border-line bg-surface space-y-2">
                {tool.guards?.requiresEnvTrue && (
                  <div className="flex items-center justify-between text-[12px]">
                    <span className="text-muted">Requires Environment Flag:</span>
                    <code className="px-2 py-0.5 rounded bg-surface-2 text-fg border border-line font-mono text-[11px]">
                      {tool.guards.requiresEnvTrue}=true
                    </code>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
