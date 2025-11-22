import React from 'react';

export interface BadgeProps {
  children: React.ReactNode;
  variant?: 'success' | 'warning' | 'error' | 'info';
}

export function Badge({ children, variant = 'info' }: BadgeProps) {
  const colors = {
    success: { bg: '#10b981', color: '#fff' },
    warning: { bg: '#f59e0b', color: '#fff' },
    error: { bg: '#ef4444', color: '#fff' },
    info: { bg: '#3b82f6', color: '#fff' },
  };

  const style = colors[variant];

  return (
    <span
      style={{
        display: 'inline-block',
        padding: '0.25rem 0.75rem',
        borderRadius: '9999px',
        fontSize: '0.875rem',
        fontWeight: 500,
        backgroundColor: style.bg,
        color: style.color,
      }}
    >
      {children}
    </span>
  );
}
