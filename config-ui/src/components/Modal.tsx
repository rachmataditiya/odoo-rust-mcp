import React, { useEffect } from 'react';
import { X } from 'lucide-react';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: React.ReactNode;
  description?: React.ReactNode;
  children: React.ReactNode;
  maxWidth?: string;
}

export function Modal({
  isOpen,
  onClose,
  title,
  description,
  children,
  maxWidth = 'max-w-xl',
}: ModalProps) {
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6 overflow-y-auto">
      <div
        className="fixed inset-0 bg-black/50 backdrop-blur-sm transition-opacity"
        onClick={onClose}
        aria-hidden="true"
      />
      <div
        role="dialog"
        aria-modal="true"
        className={`relative w-full ${maxWidth} overflow-hidden rounded-xl border border-line bg-surface shadow-2xl transition-all animate-in zoom-in-95 fade-in duration-150 my-8`}
      >
        <header className="flex items-center justify-between gap-3 border-b border-line bg-surface-2/40 px-5 py-3.5">
          <div className="min-w-0">
            <h3 className="text-[14px] font-semibold text-fg tracking-tight">
              {title}
            </h3>
            {description && (
              <p className="mt-0.5 text-[12px] text-muted">{description}</p>
            )}
          </div>
          <button
            type="button"
            onClick={onClose}
            className="grid h-7 w-7 place-items-center rounded-md text-muted transition-colors hover:bg-surface-3 hover:text-fg focus-visible:outline-2 focus-visible:outline-primary"
            aria-label="Close dialog"
          >
            <X size={15} />
          </button>
        </header>
        <div className="p-5">{children}</div>
      </div>
    </div>
  );
}
