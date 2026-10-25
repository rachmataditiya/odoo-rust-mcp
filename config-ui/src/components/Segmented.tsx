import React from 'react';

export interface SegmentedOption<T extends string | number | boolean> {
  label: React.ReactNode;
  value: T;
  blocked?: string | null;
}

export interface SegmentedProps<T extends string | number | boolean> {
  label?: string;
  options: SegmentedOption<T>[];
  value: T | null;
  onChange: (value: T) => void;
  onBlocked?: (reason: string) => void;
  disabled?: boolean;
  className?: string;
}

export function Segmented<T extends string | number | boolean>({
  label,
  options,
  value,
  onChange,
  onBlocked,
  disabled = false,
  className = '',
}: SegmentedProps<T>) {
  const activeIdx = options.findIndex((o) => o.value === value);

  return (
    <div className={`flex flex-col gap-1.5 ${disabled ? 'opacity-45 pointer-events-none' : ''} ${className}`}>
      {label && (
        <span className="text-[11px] font-medium tracking-wide text-faint uppercase">
          {label}
        </span>
      )}

      <div
        className="relative grid gap-0 rounded-lg border border-line bg-surface-2 p-1"
        style={{ gridTemplateColumns: `repeat(${options.length}, minmax(0, 1fr))` }}
      >
        {activeIdx >= 0 && (
          <div
            className="pointer-events-none absolute top-1 bottom-1 rounded-md bg-primary shadow-sm transition-all duration-200 ease-out"
            style={{
              width: `calc((100% - 0.5rem) / ${options.length})`,
              transform: `translateX(${activeIdx * 100}%)`,
            }}
          />
        )}

        {options.map((option) => {
          const isSelected = option.value === value;
          return (
            <button
              key={String(option.value)}
              type="button"
              disabled={disabled}
              onClick={() => {
                if (option.blocked) {
                  onBlocked?.(option.blocked);
                } else {
                  onChange(option.value);
                }
              }}
              aria-pressed={isSelected}
              title={option.blocked ?? undefined}
              className={`relative z-10 rounded-md px-2.5 py-1.5 text-[12px] font-medium tabular-nums transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:pointer-events-none ${
                isSelected
                  ? 'text-white'
                  : option.blocked
                  ? 'cursor-not-allowed text-faint/50 line-through'
                  : 'text-muted hover:text-fg'
              }`}
            >
              {option.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}
