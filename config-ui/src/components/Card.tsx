import React from 'react';

export interface CardProps {
  children: React.ReactNode;
  title?: React.ReactNode;
  description?: React.ReactNode;
  aside?: React.ReactNode;
  divided?: boolean;
  className?: string;
  bodyClassName?: string;
}

export function Card({
  children,
  title,
  description,
  aside,
  divided = false,
  className = '',
  bodyClassName = '',
}: CardProps) {
  return (
    <section
      className={`overflow-hidden rounded-xl border border-line bg-surface shadow-sm transition-all ${className}`}
    >
      {(title || description || aside) && (
        <header className="flex items-center justify-between gap-3 border-b border-line bg-surface-2/40 px-5 py-3.5">
          <div className="min-w-0">
            {title && (
              <h2 className="text-[14px] font-semibold text-fg tracking-tight">
                {title}
              </h2>
            )}
            {description && (
              <p className="mt-0.5 text-[12px] text-muted">{description}</p>
            )}
          </div>
          {aside && <div className="shrink-0">{aside}</div>}
        </header>
      )}
      <div className={divided ? `divide-y divide-line ${bodyClassName}` : `p-5 ${bodyClassName}`}>
        {children}
      </div>
    </section>
  );
}
