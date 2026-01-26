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
    <div className={`bg-gradient-to-br from-slate-800 to-slate-900 rounded-lg border border-slate-700 p-6 transition-all hover:border-slate-600 hover:shadow-lg hover:shadow-blue-900/20 ${className}`}>
      {title && (
        <div className="mb-6 pb-4 border-b border-slate-700/50">
          <h3 className="text-lg font-bold bg-gradient-to-r from-slate-100 to-slate-300 bg-clip-text text-transparent">{title}</h3>
          {description && (
            <p className="text-sm text-slate-400 mt-2">{description}</p>
          )}
        </div>
      )}
      {children}
    </div>
  );
};
