import React from 'react';

export interface FieldProps {
  label: React.ReactNode;
  help?: React.ReactNode;
  htmlFor?: string;
  stacked?: boolean;
  disabled?: boolean;
  children: React.ReactNode;
  className?: string;
}

export function Field({
  label,
  help,
  htmlFor,
  stacked = false,
  disabled = false,
  children,
  className = '',
}: FieldProps) {
  return (
    <div
      className={`flex gap-x-6 gap-y-2 px-5 py-3.5 transition-opacity ${
        stacked
          ? 'flex-col'
          : 'flex-col sm:flex-row sm:items-center sm:justify-between'
      } ${disabled ? 'opacity-45 pointer-events-none' : ''} ${className}`}
    >
      <div className="min-w-0 sm:max-w-[55%]">
        <label
          htmlFor={htmlFor}
          className="block text-[13px] font-medium text-fg"
        >
          {label}
        </label>
        {help && (
          <p className="mt-0.5 text-[12px] leading-snug text-muted">{help}</p>
        )}
      </div>
      <div
        className={`flex shrink-0 items-center gap-2 ${
          stacked ? '' : 'sm:justify-end'
        }`}
      >
        {children}
      </div>
    </div>
  );
}
