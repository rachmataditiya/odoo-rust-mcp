export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  id?: string;
  label?: string;
  disabled?: boolean;
  busy?: boolean;
  className?: string;
}

export function Switch({
  checked,
  onChange,
  id,
  label,
  disabled = false,
  busy = false,
  className = '',
}: SwitchProps) {
  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled || busy}
      aria-busy={busy}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:pointer-events-none disabled:opacity-45 ${
        checked ? 'bg-primary' : 'bg-surface-3'
      } ${className}`}
    >
      <span
        className={`inline-flex h-4 w-4 items-center justify-center rounded-full bg-white shadow-sm transition-transform duration-150 ${
          checked ? 'translate-x-4.5' : 'translate-x-0.5'
        }`}
      >
        {busy && (
          <span className="h-2.5 w-2.5 animate-spin rounded-full border-[1.5px] border-primary border-t-transparent" />
        )}
      </span>
    </button>
  );
}
