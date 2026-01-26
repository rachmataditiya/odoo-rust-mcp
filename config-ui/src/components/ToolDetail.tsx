import React, { useState } from 'react';
import { ChevronDown, ChevronRight, Shield, Settings } from 'lucide-react';
import type { ToolConfig } from '../types';

interface ToolDetailProps {
  tool: ToolConfig;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onToggleGuard?: (guardKey: string, enabled: boolean) => void;
  availableGuards?: Array<{ key: string; value: string; label: string }>;
}

export function ToolDetail({ tool, enabled, onToggle, onToggleGuard, availableGuards = [] }: ToolDetailProps) {
  const [expanded, setExpanded] = useState(false);

  const inputSchema = tool.inputSchema || {};
  const properties = inputSchema.properties || {};
  const required = inputSchema.required || [];
  const hasGuards = tool.guards && Object.keys(tool.guards).length > 0;

  return (
    <div className={`border rounded-lg transition-all ${
      enabled ? 'border-blue-300 bg-blue-50' : 'border-gray-200 bg-white'
    }`}>
      <div className="p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <button
                onClick={() => setExpanded(!expanded)}
                className="flex items-center gap-1 hover:text-blue-600 transition-colors"
              >
                {expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}
                <h3 className="font-mono text-sm font-semibold text-gray-900">
                  {tool.name}
                </h3>
              </button>
              {hasGuards && (
                <Shield size={14} className="text-orange-600" title="Has security guards" />
              )}
            </div>
            <p className="text-sm text-gray-600 leading-relaxed">
              {tool.description || 'No description available'}
            </p>
          </div>

          <button
            onClick={() => onToggle(!enabled)}
            className={`relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors ${
              enabled ? 'bg-blue-600' : 'bg-gray-300'
            }`}
            title={enabled ? 'Disable tool' : 'Enable tool'}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                enabled ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {expanded && (
          <div className="mt-4 space-y-4 pt-4 border-t border-gray-200">
            {(hasGuards || availableGuards.length > 0) && (
              <div className="bg-orange-50 border border-orange-200 rounded-lg p-3">
                <div className="flex items-center gap-2 mb-3">
                  <Shield size={14} className="text-orange-600" />
                  <span className="text-xs font-semibold text-orange-900">Security Guards</span>
                </div>

                {availableGuards.length > 0 && onToggleGuard ? (
                  <div className="space-y-2">
                    {availableGuards.map(({ key, value, label }) => {
                      const isActive = tool.guards?.[key] === value;
                      return (
                        <div
                          key={`${key}-${value}`}
                          className={`flex items-center justify-between p-2 rounded border transition-colors ${
                            isActive
                              ? 'bg-orange-100 border-orange-300'
                              : 'bg-white border-orange-200'
                          }`}
                        >
                          <div className="flex-1 min-w-0">
                            <p className="text-xs font-medium text-orange-900">{label}</p>
                            <p className="text-[10px] text-orange-700 font-mono mt-0.5">
                              {key}: {value}
                            </p>
                          </div>
                          <button
                            onClick={() => onToggleGuard(key, !isActive)}
                            className={`relative inline-flex h-5 w-9 flex-shrink-0 items-center rounded-full transition-colors ml-3 ${
                              isActive ? 'bg-orange-600' : 'bg-gray-300'
                            }`}
                            title={isActive ? 'Remove guard' : 'Add guard'}
                          >
                            <span
                              className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${
                                isActive ? 'translate-x-5' : 'translate-x-1'
                              }`}
                            />
                          </button>
                        </div>
                      );
                    })}
                  </div>
                ) : hasGuards ? (
                  <div className="space-y-1">
                    {Object.entries(tool.guards).map(([key, value]) => (
                      <div key={key} className="text-xs">
                        <span className="font-mono text-orange-700">{key}:</span>{' '}
                        <span className="text-orange-600">{String(value)}</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-xs text-orange-700">No guards configured</p>
                )}
              </div>
            )}

            {tool.op && (
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-3">
                <div className="flex items-center gap-2 mb-2">
                  <Settings size={14} className="text-gray-600" />
                  <span className="text-xs font-semibold text-gray-900">Operation</span>
                </div>
                <div className="space-y-1">
                  <div className="text-xs">
                    <span className="font-mono text-gray-700">type:</span>{' '}
                    <span className="text-gray-600">{tool.op.type}</span>
                  </div>
                  {tool.op.map && (
                    <div className="text-xs mt-2">
                      <span className="font-mono text-gray-700">mapping:</span>
                      <div className="mt-1 pl-3 space-y-0.5">
                        {Object.entries(tool.op.map).map(([key, value]) => (
                          <div key={key} className="font-mono text-[11px] text-gray-600">
                            {key} → {String(value)}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {Object.keys(properties).length > 0 && (
              <div>
                <h4 className="text-xs font-semibold text-gray-900 mb-2">Parameters</h4>
                <div className="space-y-2">
                  {Object.entries(properties).map(([paramName, paramSchema]: [string, any]) => {
                    const isRequired = required.includes(paramName);
                    return (
                      <div
                        key={paramName}
                        className={`p-3 rounded-lg border ${
                          isRequired
                            ? 'border-blue-200 bg-blue-50'
                            : 'border-gray-200 bg-gray-50'
                        }`}
                      >
                        <div className="flex items-start justify-between gap-2 mb-1">
                          <span className="font-mono text-xs font-semibold text-gray-900">
                            {paramName}
                          </span>
                          {isRequired && (
                            <span className="text-[10px] px-1.5 py-0.5 bg-blue-600 text-white rounded font-medium">
                              required
                            </span>
                          )}
                        </div>

                        <div className="space-y-1">
                          <div className="flex items-center gap-2">
                            <span className="text-[10px] text-gray-500">Type:</span>
                            <span className="text-xs font-mono text-gray-700">
                              {paramSchema.type || 'any'}
                              {paramSchema.items && (
                                <span className="text-gray-500">
                                  {'<'}
                                  {paramSchema.items.type || 'any'}
                                  {'>'}
                                </span>
                              )}
                            </span>
                          </div>

                          {paramSchema.description && (
                            <p className="text-xs text-gray-600 leading-relaxed">
                              {paramSchema.description}
                            </p>
                          )}

                          {paramSchema.enum && (
                            <div className="mt-1">
                              <span className="text-[10px] text-gray-500">Values:</span>
                              <div className="flex flex-wrap gap-1 mt-1">
                                {paramSchema.enum.map((val: string) => (
                                  <span
                                    key={val}
                                    className="text-[10px] px-1.5 py-0.5 bg-gray-200 text-gray-700 rounded font-mono"
                                  >
                                    {val}
                                  </span>
                                ))}
                              </div>
                            </div>
                          )}

                          {paramSchema.default !== undefined && (
                            <div className="text-xs text-gray-600">
                              <span className="text-gray-500">Default:</span>{' '}
                              <span className="font-mono">{JSON.stringify(paramSchema.default)}</span>
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
