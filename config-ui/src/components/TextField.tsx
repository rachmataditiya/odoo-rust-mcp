import React, { useState } from 'react';
import { Eye, EyeOff } from 'lucide-react';

export interface TextFieldProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'prefix'> {
  secret?: boolean;
  mono?: boolean;
  icon?: React.ReactNode;
  clearable?: boolean;
  onClear?: () => void;
  containerClassName?: string;
}

export function TextField({
  secret = false,
  mono = false,
  icon,
  type = 'text',
  className = '',
  containerClassName = '',
  value,
  ...props
}: TextFieldProps) {
  const [reveal, setReveal] = useState(false);

  return (
    <div className={`relative w-full ${containerClassName}`}>
      {icon && (
        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-muted">
          {icon}
        </div>
      )}
      <input
        autoComplete="off"
        autoCapitalize="off"
        spellCheck="false"
        type={secret ? (reveal ? 'text' : 'password') : type}
        value={value}
        className={`w-full rounded-md border border-line bg-surface-2 px-3 py-1.5 text-[13px] text-fg transition-colors outline-none placeholder:text-faint hover:border-line-strong focus:border-primary focus:ring-1 focus:ring-primary disabled:pointer-events-none disabled:opacity-45 ${
          icon ? 'pl-9' : ''
        } ${secret ? 'pr-9' : ''} ${mono ? 'font-mono' : ''} ${className}`}
        {...props}
      />
      {secret && (
        <button
          type="button"
          tabIndex={-1}
          onClick={() => setReveal(!reveal)}
          aria-label={reveal ? 'Hide password' : 'Show password'}
          title={reveal ? 'Hide password' : 'Show password'}
          className="absolute inset-y-0 right-0 grid w-9 place-items-center text-faint transition-colors hover:text-fg focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-primary"
        >
          {reveal ? <EyeOff size={14} /> : <Eye size={14} />}
        </button>
      )}
    </div>
  );
}
