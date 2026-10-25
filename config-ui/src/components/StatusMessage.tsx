import { CheckCircle, AlertCircle, AlertTriangle, Info, Loader2, X } from 'lucide-react';
import type { StatusMessage as StatusMessageType } from '../types';

interface StatusMessageProps {
  status: StatusMessageType;
  onDismiss?: () => void;
}

export function StatusMessage({ status, onDismiss }: StatusMessageProps) {
  const variants = {
    loading: {
      icon: Loader2,
      bgColor: 'bg-primary/10',
      borderColor: 'border-primary/20',
      textColor: 'text-fg',
      iconColor: 'text-primary',
    },
    success: {
      icon: CheckCircle,
      bgColor: 'bg-ok/10',
      borderColor: 'border-ok/25',
      textColor: 'text-fg',
      iconColor: 'text-ok',
    },
    warning: {
      icon: AlertTriangle,
      bgColor: 'bg-warn/10',
      borderColor: 'border-warn/25',
      textColor: 'text-fg',
      iconColor: 'text-warn',
    },
    error: {
      icon: AlertCircle,
      bgColor: 'bg-down/10',
      borderColor: 'border-down/25',
      textColor: 'text-fg',
      iconColor: 'text-down',
    },
    info: {
      icon: Info,
      bgColor: 'bg-primary/10',
      borderColor: 'border-primary/20',
      textColor: 'text-fg',
      iconColor: 'text-primary',
    },
  };

  const config = variants[status.type] || variants.info;
  const Icon = config.icon;

  return (
    <div
      className={`flex items-start gap-3 p-3.5 rounded-xl border ${config.bgColor} ${config.borderColor} ${config.textColor} animate-in slide-in-from-top-2 duration-200 shadow-sm`}
    >
      <Icon
        className={`${config.iconColor} shrink-0 mt-0.5 ${
          status.type === 'loading' ? 'animate-spin' : ''
        }`}
        size={16}
      />
      <p className="flex-1 text-[13px] font-medium leading-snug">{status.message}</p>
      {(status.type === 'error' || status.type === 'warning' || status.type === 'info') &&
        onDismiss && (
          <button
            type="button"
            onClick={onDismiss}
            className="shrink-0 text-muted hover:text-fg transition-colors"
            aria-label="Dismiss"
          >
            <X size={15} />
          </button>
        )}
    </div>
  );
}
