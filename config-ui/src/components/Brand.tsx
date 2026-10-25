import packageJson from '../../package.json';

interface BrandProps {
  small?: boolean;
}

export function Brand({ small = false }: BrandProps) {
  return (
    <div
      className={`flex items-center font-semibold tracking-tight text-fg ${
        small ? 'gap-2.5 text-[15px]' : 'gap-3 text-lg'
      }`}
    >
      <div className="relative flex items-center justify-center shrink-0">
        <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-primary to-blue-600 flex items-center justify-center shadow-sm shadow-primary/20 text-white">
          <svg
            className="w-4 h-4 fill-current"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            {/* Elegant geometric Peugeot 3008 inspired badge / MCP node */}
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" fill="none" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col leading-tight min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="font-semibold text-fg tracking-tight">Odoo MCP</span>
          <span className="text-[10px] font-medium px-1.5 py-0.5 rounded-full bg-surface-2 text-muted border border-line">
            v{packageJson.version}
          </span>
        </div>
        <span className="text-[11px] font-normal text-muted truncate">
          Peugeot 3008 Edition
        </span>
      </div>
    </div>
  );
}
