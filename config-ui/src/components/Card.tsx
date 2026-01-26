import React from 'react';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  title?: string;
  description?: string;
}

export const Card: React.FC<CardProps> = ({ 
  children, 
  className = '', 
  title,
  description 
}) => {
  return (
    <div className={`bg-slate-800 rounded-lg border border-slate-700 p-6 transition-all hover:border-slate-600 ${className}`}>
      {title && (
        <div className="mb-4">
          <h3 className="text-lg font-semibold text-slate-100">{title}</h3>
          {description && (
            <p className="text-sm text-slate-400 mt-1">{description}</p>
          )}
        </div>
      )}
      {children}
    </div>
  );
};
