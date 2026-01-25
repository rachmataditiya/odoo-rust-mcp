import React from 'react';
import type { StatusMessage as StatusMessageType } from '../types';

interface StatusMessageProps {
  status: StatusMessageType | null;
}

export const StatusMessage: React.FC<StatusMessageProps> = ({ status }) => {
  if (!status) return null;

  const bgColors = {
    success: 'bg-green-50 border-green-200 text-green-800',
    error: 'bg-red-50 border-red-200 text-red-800',
    loading: 'bg-blue-50 border-blue-200 text-blue-800',
  };

  return (
    <div className={`mt-4 p-3 rounded border ${bgColors[status.type]}`}>
      {status.message}
    </div>
  );
};
