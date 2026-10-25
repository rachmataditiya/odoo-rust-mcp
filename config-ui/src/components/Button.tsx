import React from 'react';
import { Loader2 } from 'lucide-react';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'plain';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
  icon?: React.ReactNode;
}

export function Button({
  children,
  variant = 'primary',
  size = 'md',
  loading = false,
  icon,
  disabled,
  className = '',
  ...props
}: ButtonProps) {
  const baseStyles =
    'inline-flex items-center justify-center font-medium rounded-lg transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:pointer-events-none disabled:opacity-45';

  const variants = {
    primary: 'bg-primary text-white hover:bg-primary-hover shadow-sm',
    secondary: 'border border-line bg-surface-2 text-muted hover:bg-surface-3 hover:text-fg',
    plain: 'border border-line bg-surface-2 text-muted hover:bg-surface-3 hover:text-fg',
    ghost: 'text-muted hover:bg-surface-2 hover:text-fg',
    danger: 'border border-line bg-surface-2 text-down hover:border-down hover:bg-down hover:text-white',
  };

  const sizes = {
    sm: 'px-2.5 py-1 text-[12px] gap-1.5',
    md: 'px-3 py-1.5 text-[13px] gap-2',
    lg: 'px-4 py-2.5 text-sm gap-2.5',
  };

  return (
    <button
      className={`${baseStyles} ${variants[variant]} ${sizes[size]} ${className}`}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? (
        <Loader2 className="animate-spin" size={size === 'sm' ? 13 : size === 'lg' ? 18 : 15} />
      ) : (
        icon
      )}
      {children}
    </button>
  );
}
