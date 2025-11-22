import React from 'react';

export interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
  disabled?: boolean;
}

export function Button({ children, onClick, variant = 'primary', disabled = false }: ButtonProps) {
  const colors = {
    primary: '#0066cc',
    secondary: '#6c757d',
    danger: '#dc3545',
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: '0.5rem 1rem',
        border: 'none',
        borderRadius: '4px',
        fontSize: '1rem',
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.6 : 1,
        backgroundColor: colors[variant],
        color: 'white',
        fontWeight: 500,
        transition: 'opacity 0.2s',
      }}
    >
      {children}
    </button>
  );
}
