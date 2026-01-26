import React from 'react';
import type { StatusMessage as StatusMessageType } from '../types';

interface StatusMessageProps {
  status: StatusMessageType | null;
}

export const StatusMessage: React.FC<StatusMessageProps> = ({ status }) => {
  if (!status) return null;

  const bgColors = {
    success: 'bg-green-900/30 border-green-700 text-green-300',
    error: 'bg-red-900/30 border-red-700 text-red-300',
    loading: 'bg-blue-900/30 border-blue-700 text-blue-300',
  };

  const icons = {
    success: '✓',
    error: '✕',
    loading: '⟳',
  };

  return (
    <div className={`p-3 rounded border ${bgColors[status.type]} flex items-center gap-2 text-sm`}>
      <span className="flex-shrink-0">{icons[status.type]}</span>
      <span>{status.message}</span>
    </div>
  );
};
